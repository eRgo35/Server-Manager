use crate::model::Stats;

#[derive(Debug, thiserror::Error)]
pub enum StatsError {
    #[error("malformed stats output: {0}")]
    Malformed(String),
}

fn section<'a>(raw: &'a str, name: &str) -> Result<&'a str, StatsError> {
    let start = raw
        .find(&format!("---SM-{name}\n"))
        .ok_or_else(|| StatsError::Malformed(format!("missing section {name}")))?;
    let rest = &raw[start + name.len() + 7..];
    let end = rest.find("---SM-").unwrap_or(rest.len());
    Ok(rest[..end].trim())
}

fn cpu_busy_total(line: &str) -> Result<(u64, u64), StatsError> {
    let nums: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|t| t.parse().ok())
        .collect();
    if nums.len() < 4 {
        return Err(StatsError::Malformed("cpu line".into()));
    }
    let total: u64 = nums.iter().sum();
    let idle = nums[3];
    Ok((total - idle, total))
}

pub fn parse_proc_stats(raw: &str) -> Result<Stats, StatsError> {
    let uptime_secs = section(raw, "UPTIME")?
        .split_whitespace()
        .next()
        .and_then(|f| f.parse::<f64>().ok())
        .ok_or_else(|| StatsError::Malformed("uptime".into()))? as u64;

    let mem = section(raw, "MEM")?;
    let kb = |key: &str| {
        mem.lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<u64>().ok())
            .map(|v| v * 1024)
    };
    let mem_total = kb("MemTotal:").ok_or_else(|| StatsError::Malformed("MemTotal".into()))?;
    let mem_avail =
        kb("MemAvailable:").ok_or_else(|| StatsError::Malformed("MemAvailable".into()))?;
    let mem_used = mem_total.saturating_sub(mem_avail);

    let disk_line = section(raw, "DISK")?;
    let mut dit = disk_line.split_whitespace();
    let disk_total = dit
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| StatsError::Malformed("disk size".into()))?;
    let disk_used = dit
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| StatsError::Malformed("disk used".into()))?;

    let (b1, t1) = cpu_busy_total(section(raw, "CPU1")?)?;
    let (b2, t2) = cpu_busy_total(section(raw, "CPU2")?)?;
    let dtotal = t2.saturating_sub(t1).max(1);
    let dbusy = b2.saturating_sub(b1);
    let cpu_pct = (100.0 * dbusy as f64 / dtotal as f64) as f32;

    Ok(Stats {
        cpu_pct,
        mem_used,
        mem_total,
        disk_used,
        disk_total,
        uptime_secs,
    })
}

/// The remote command whose output `parse_proc_stats` consumes.
pub const PROC_STATS_CMD: &str = "echo '---SM-UPTIME'; cat /proc/uptime; echo '---SM-MEM'; cat /proc/meminfo; echo '---SM-DISK'; df -B1 --output=size,used / | tail -1; echo '---SM-CPU1'; head -1 /proc/stat; sleep 0.2; echo '---SM-CPU2'; head -1 /proc/stat";

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &str = "\
---SM-UPTIME
12345.67 98765.43
---SM-MEM
MemTotal:       16384000 kB
MemFree:         1000000 kB
MemAvailable:    8192000 kB
Buffers:          200000 kB
---SM-DISK
   500107862016   123456789012
---SM-CPU1
cpu  100 0 100 800 0 0 0 0 0 0
---SM-CPU2
cpu  110 0 110 860 0 0 0 0 0 0
";

    #[test]
    fn parses_all_fields() {
        let s = parse_proc_stats(RAW).unwrap();
        assert_eq!(s.uptime_secs, 12345);
        assert_eq!(s.mem_total, 16_384_000 * 1024);
        assert_eq!(s.mem_used, (16_384_000 - 8_192_000) * 1024);
        assert_eq!(s.disk_total, 500_107_862_016);
        assert_eq!(s.disk_used, 123_456_789_012);
        // CPU1: total=1000, idle=800, busy=200. CPU2: total=1080, idle=860, busy=220.
        // dtotal=80, dbusy=20 -> pct = 100*20/80 = 25.0
        assert!((s.cpu_pct - 25.0).abs() < 0.1);
    }

    #[test]
    fn malformed_errs() {
        assert!(parse_proc_stats("nonsense").is_err());
    }
}
