//! Parser for `virsh nodecpustats` and `virsh nodememstats` (remote host metrics).

/// Parsed host memory metrics from `virsh nodememstats`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeMemStats {
    pub total_kib: u64,
    pub free_kib: u64,
    pub available_kib: u64,
    pub buffers_kib: u64,
    pub cached_kib: u64,
}

/// Parse `virsh nodecpustats --percent` output into an overall CPU busy fraction (0.0 ..= 100.0).
///
/// Typical output format:
/// ```text
/// usage:           12.3%
/// user:             8.1%
/// system:           4.2%
/// idle:            87.7%
/// iowait:           0.0%
/// ```
pub fn parse_nodecpustats_percent(text: &str) -> Option<f64> {
    let mut usage = None;
    let mut idle = None;
    for line in text.lines() {
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let val_str = val.trim().trim_end_matches('%').trim();
        let Ok(pct) = val_str.parse::<f64>() else {
            continue;
        };
        match key {
            "usage" => usage = Some(pct),
            "idle" => idle = Some(pct),
            _ => {}
        }
    }
    usage
        .or_else(|| idle.map(|i| (100.0 - i).max(0.0)))
        .map(|p| p.clamp(0.0, 100.0))
}

/// Parse `virsh nodememstats` output into [`NodeMemStats`].
///
/// Typical output format:
/// ```text
/// total    :   8067340 KiB
/// free     :    628472 KiB
/// available:   4726880 KiB
/// buffers  :    118296 KiB
/// cached   :   4127836 KiB
/// ```
pub fn parse_nodememstats(text: &str) -> Option<NodeMemStats> {
    let mut stats = NodeMemStats::default();
    let mut found_any = false;
    let mut has_available = false;

    for line in text.lines() {
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let val_num = val
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        match key {
            "total" => {
                stats.total_kib = val_num;
                found_any = true;
            }
            "free" => {
                stats.free_kib = val_num;
                found_any = true;
            }
            "available" => {
                stats.available_kib = val_num;
                has_available = true;
                found_any = true;
            }
            "buffers" => {
                stats.buffers_kib = val_num;
                found_any = true;
            }
            "cached" => {
                stats.cached_kib = val_num;
                found_any = true;
            }
            _ => {}
        }
    }

    if !found_any {
        return None;
    }

    if !has_available {
        stats.available_kib = stats
            .free_kib
            .saturating_add(stats.buffers_kib)
            .saturating_add(stats.cached_kib);
    }

    Some(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nodememstats_fixture() {
        let text = include_str!("../../../tests/fixtures/virsh/nodememstats.txt");
        let stats = parse_nodememstats(text).expect("should parse nodememstats fixture");
        assert_eq!(stats.total_kib, 8067340);
        assert_eq!(stats.free_kib, 628472);
        assert_eq!(stats.available_kib, 4726880);
        assert_eq!(stats.buffers_kib, 118296);
        assert_eq!(stats.cached_kib, 4127836);
    }

    #[test]
    fn parses_nodememstats_without_available() {
        let text = "total: 1000 KiB\nfree: 200 KiB\nbuffers: 100 KiB\ncached: 300 KiB\n";
        let stats = parse_nodememstats(text).expect("should parse");
        assert_eq!(stats.total_kib, 1000);
        assert_eq!(stats.free_kib, 200);
        assert_eq!(stats.available_kib, 600); // 200 + 100 + 300
    }

    #[test]
    fn parses_nodecpustats_usage() {
        let text = "usage: 24.5%\nuser: 14.2%\nsystem: 10.3%\nidle: 75.5%\n";
        let pct = parse_nodecpustats_percent(text).expect("should parse");
        assert!((pct - 24.5).abs() < 0.001);
    }

    #[test]
    fn parses_nodecpustats_idle_fallback() {
        let text = "user: 10.0%\nsystem: 5.0%\nidle: 80.0%\n";
        let pct = parse_nodecpustats_percent(text).expect("should parse");
        assert!((pct - 20.0).abs() < 0.001);
    }
}
