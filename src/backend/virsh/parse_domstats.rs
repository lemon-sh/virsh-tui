//! Parser for `virsh domstats --raw`.

use super::super::RawDomainStats;
use color_eyre::Result;

/// Parse `virsh domstats --raw` into per-domain key/value maps.
pub fn parse_domstats_raw(text: &str) -> Result<Vec<RawDomainStats>> {
    let mut out: Vec<RawDomainStats> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix("Domain: '").and_then(|s| s.strip_suffix('\'')) {
            out.push(RawDomainStats {
                domain: name.to_string(),
                values: std::collections::HashMap::new(),
            });
        } else if let Some(current) = out.last_mut() {
            let trimmed = line.trim();
            if let Some((k, v)) = trimmed.split_once('=') {
                current.values.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    Ok(out)
}
