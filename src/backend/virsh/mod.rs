//! Virsh subprocess backend (read paths for P2).

pub mod events;
pub mod exec;
pub mod host_local;
pub mod parse_domstats;
pub mod parse_help;
pub mod parse_leases;
pub mod parse_list;
pub mod parse_nodestats;
pub mod parse_xml;

use super::{Backend, RawDomainStats};
use crate::model::{DhcpLease, DomainSummary, HostInfo, NetworkSummary, PoolSummary, Snapshot};
use color_eyre::Result;
pub use exec::{ensure_virsh_available, is_virsh_available};

/// Real backend: runs `virsh -c <uri> ...` with `LC_ALL=C`.
pub struct VirshBackend {
    uri: String,
}

impl VirshBackend {
    /// Create a backend for a connection URI.
    pub fn new(uri: &str) -> Self {
        Self { uri: uri.to_string() }
    }

    /// Connection URI.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Fetch domain names configured for autostart.
    pub async fn autostart_domains(&self) -> Result<std::collections::HashSet<String>> {
        let text = exec::run(&self.uri, &["list", "--all", "--autostart", "--name"]).await?;
        Ok(text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }

    /// One refresh: domain summaries plus the raw stats they were built from.
    ///
    /// `virsh domstats --raw` lists every domain (active and inactive) with its
    /// state, so a single call feeds both the table and the metrics store.
    pub async fn domains_with_stats(&self) -> Result<(Vec<DomainSummary>, Vec<RawDomainStats>)> {
        self.domains_with_stats_cached(None).await
    }

    /// One refresh: domain summaries plus raw stats, reusing a cached autostart domain set.
    ///
    /// If `cached_autostart` is `None`, queries `virsh list --all --autostart --name`.
    pub async fn domains_with_stats_cached(
        &self,
        cached_autostart: Option<&std::collections::HashSet<String>>,
    ) -> Result<(Vec<DomainSummary>, Vec<RawDomainStats>)> {
        let stats_text = exec::run(&self.uri, &["domstats", "--raw"]).await?;
        let stats = parse_domstats::parse_domstats_raw(&stats_text)?;
        let fallback_autostart;
        let autostart: &std::collections::HashSet<String> = match cached_autostart {
            Some(cached) => cached,
            None => {
                fallback_autostart = self.autostart_domains().await.unwrap_or_default();
                &fallback_autostart
            }
        };
        let summaries = stats
            .iter()
            .map(|s| DomainSummary {
                name: s.domain.clone(),
                uuid: String::new(),
                id: None,
                state: state_from_code(s.u64("state.state")),
                autostart: autostart.contains(&s.domain),
                persistent: true,
                vcpus: s.u64("vcpu.current").unwrap_or(1) as u32,
                max_vcpus: s.u64("vcpu.maximum").unwrap_or(1) as u32,
                mem_kib: s.u64("balloon.current").unwrap_or(0),
                max_mem_kib: s.u64("balloon.maximum").unwrap_or(0),
                has_managed_save: false,
            })
            .collect();
        Ok((summaries, stats))
    }

    /// Hypervisor hostname via `virsh hostname`.
    pub async fn hostname(&self) -> Result<String> {
        let text = exec::run(&self.uri, &["hostname"]).await?;
        let name = text.trim();
        if name.is_empty() {
            Ok(hostname())
        } else {
            Ok(crate::model::sanitize(name))
        }
    }

    /// Remote host CPU and memory telemetry via `nodecpustats --percent` and `nodememstats`.
    pub async fn node_stats(&self) -> Result<(Option<f64>, Option<parse_nodestats::NodeMemStats>)> {
        let (cpu_res, mem_res) = tokio::join!(
            exec::run(&self.uri, &["nodecpustats", "--percent"]),
            exec::run(&self.uri, &["nodememstats"])
        );
        let cpu = cpu_res
            .ok()
            .and_then(|t| parse_nodestats::parse_nodecpustats_percent(&t));
        let mem = mem_res.ok().and_then(|t| parse_nodestats::parse_nodememstats(&t));
        Ok((cpu, mem))
    }
}

#[async_trait::async_trait]
impl Backend for VirshBackend {
    async fn list_domains(&self) -> Result<Vec<DomainSummary>> {
        Ok(self.domains_with_stats().await?.0)
    }

    async fn all_domain_stats(&self) -> Result<Vec<RawDomainStats>> {
        let text = exec::run(&self.uri, &["domstats", "--raw"]).await?;
        Ok(parse_domstats::parse_domstats_raw(&text)?)
    }

    async fn host_info(&self) -> Result<HostInfo> {
        let nodeinfo = exec::run(&self.uri, &["nodeinfo"]).await?;
        let host_name = self.hostname().await.unwrap_or_else(|_| hostname());
        Ok(host_local::parse_nodeinfo(&nodeinfo, &host_name))
    }

    async fn networks(&self) -> Result<Vec<NetworkSummary>> {
        let text = exec::run(&self.uri, &["net-list", "--all"]).await?;
        Ok(parse_list::parse_net_list(&text)?)
    }

    async fn dhcp_leases(&self, net: &str) -> Result<Vec<DhcpLease>> {
        let text = exec::run(&self.uri, &["net-dhcp-leases", net]).await?;
        Ok(parse_leases::parse_leases(&text))
    }

    async fn pools(&self) -> Result<Vec<PoolSummary>> {
        let text = exec::run(&self.uri, &["pool-list", "--all", "--details"]).await?;
        Ok(parse_list::parse_pool_list(&text)?)
    }

    async fn snapshots(&self, domain: &str) -> Result<Vec<Snapshot>> {
        let text = exec::run(&self.uri, &["snapshot-list", domain, "--parent"])
            .await
            .unwrap_or_default();
        Ok(parse_list::parse_snapshot_list(&text))
    }
}

/// Map `state.state` (virDomainState) to the UI state.
///
/// 1 running, 2 blocked and 4 shutting-down guests are still running.
pub fn state_from_code(code: Option<u64>) -> crate::model::DomainState {
    use crate::model::DomainState as S;
    match code {
        Some(1 | 2 | 4) => S::Running,
        Some(3) => S::Paused,
        Some(5) => S::ShutOff,
        Some(6) => S::Crashed,
        Some(7) => S::PmSuspended,
        _ => S::Other,
    }
}

/// Local hostname (used in the top bar).
pub fn hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| String::from("localhost"))
}
