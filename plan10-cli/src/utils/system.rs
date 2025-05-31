use anyhow::Result;
use std::process::Command;
use sysinfo::{System, SystemExt, CpuExt, DiskExt};

pub struct SystemInfo {
    pub hostname: String,
    pub uptime: u64,
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_available: u64,
    pub load_average: (f64, f64, f64),
    pub disks: Vec<DiskInfo>,
}

pub struct DiskInfo {
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: u8,
}

pub fn get_system_info() -> Result<SystemInfo> {
    let mut system = System::new_all();
    system.refresh_all();

    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let load_avg = system.load_average();
    
    let mut disks = Vec::new();
    for disk in system.disks() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let usage_percent = if total > 0 { 
            ((used as f64 / total as f64) * 100.0) as u8 
        } else { 
            0 
        };

        disks.push(DiskInfo {
            mount_point: disk.mount_point().display().to_string(),
            total_space: total,
            available_space: available,
            used_space: used,
            usage_percent,
        });
    }

    Ok(SystemInfo {
        hostname,
        uptime: system.uptime(),
        cpu_usage: system.global_cpu_info().cpu_usage(),
        memory_total: system.total_memory(),
        memory_used: system.used_memory(),
        memory_available: system.available_memory(),
        load_average: (load_avg.one, load_avg.five, load_avg.fifteen),
        disks,
    })
}

pub fn get_macos_version() -> Result<String> {
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()?;
    
    if output.status.success() {
        let stdout_string = String::from_utf8_lossy(&output.stdout);
        Ok(stdout_string.trim().to_string())
    } else {
        Ok("Unknown".to_string())
    }
}

pub fn get_uptime_string() -> Result<String> {
    let output = Command::new("uptime")
        .output()?;
    
    if output.status.success() {
        let stdout_string = String::from_utf8_lossy(&output.stdout);
        Ok(stdout_string.trim().to_string())
    } else {
        Ok("Unknown".to_string())
    }
}

pub fn get_thermal_state() -> Result<String> {
    let output = Command::new("pmset")
        .args(&["-g", "therm"])
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok("Unable to get thermal state".to_string())
    }
}

pub fn is_on_battery() -> Result<bool> {
    let output = Command::new("pmset")
        .args(&["-g", "batt"])
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("Battery Power"))
    } else {
        Ok(false)
    }
}

pub fn is_on_ac_power() -> Result<bool> {
    let output = Command::new("pmset")
        .args(&["-g", "batt"])
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("AC Power"))
    } else {
        Ok(false)
    }
}

pub fn get_battery_percentage() -> Result<Option<u8>> {
    let output = Command::new("pmset")
        .args(&["-g", "batt"])
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(start) = line.find(char::is_numeric) {
                if let Some(end) = line[start..].find('%') {
                    let percentage_str = &line[start..start + end];
                    if let Ok(percentage) = percentage_str.parse::<u8>() {
                        return Ok(Some(percentage));
                    }
                }
            }
        }
    }
    
    Ok(None)
}

pub fn is_caffeinate_running() -> Result<bool> {
    let output = Command::new("pgrep")
        .args(&["-x", "caffeinate"])
        .output()?;
    
    Ok(output.status.success() && !output.stdout.is_empty())
}

pub fn get_caffeinate_pids() -> Result<Vec<u32>> {
    let output = Command::new("pgrep")
        .arg("caffeinate")
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pids: Vec<u32> = stdout
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect();
        Ok(pids)
    } else {
        Ok(vec![])
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn format_duration_seconds(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}