use colored::*;
use std::fmt;

pub fn format_status_icon(status: &str) -> ColoredString {
    match status.to_lowercase().as_str() {
        "good" | "normal" | "ok" | "healthy" => "✅".green(),
        "warning" | "medium" | "moderate" => "⚠️".yellow(),
        "error" | "critical" | "bad" | "high" => "❌".red(),
        "unknown" | "unavailable" => "❓".dimmed(),
        _ => "ℹ️".blue(),
    }
}

pub fn format_percentage_status(percentage: u8) -> (ColoredString, &'static str) {
    match percentage {
        81..=100 => ("🟢".green(), "Good"),
        51..=80 => ("🟡".yellow(), "Medium"),
        21..=50 => ("🟠".yellow(), "Low"),
        _ => ("🔴".red(), "Critical"),
    }
}

pub fn format_temperature_status(temp_celsius: f32) -> (ColoredString, &'static str) {
    match temp_celsius {
        t if t < 60.0 => ("❄️".blue(), "Cool"),
        t if t < 75.0 => ("🌡️".green(), "Normal"),
        t if t < 85.0 => ("🔶".yellow(), "Warm"),
        _ => ("🔥".red(), "Hot"),
    }
}

pub fn format_power_source(on_battery: bool, on_ac: bool) -> ColoredString {
    if on_ac {
        "🔌 AC Power".green()
    } else if on_battery {
        "🔋 Battery Power".yellow()
    } else {
        "❓ Unknown Power Source".red()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
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

pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn format_time_remaining(minutes: Option<u32>) -> String {
    match minutes {
        Some(mins) => {
            let hours = mins / 60;
            let remaining_mins = mins % 60;
            if hours > 0 {
                format!("{}:{:02}", hours, remaining_mins)
            } else {
                format!("0:{:02}", remaining_mins)
            }
        },
        None => "Unknown".to_string(),
    }
}

pub fn format_cpu_usage(usage: f32) -> ColoredString {
    let usage_str = format!("{:.1}%", usage);
    match usage {
        u if u < 25.0 => usage_str.green(),
        u if u < 50.0 => usage_str.yellow(),
        u if u < 75.0 => usage_str.red(),
        _ => usage_str.bright_red().bold(),
    }
}

pub fn format_memory_usage(used: u64, total: u64) -> String {
    let usage_percent = if total > 0 {
        (used as f64 / total as f64 * 100.0) as u8
    } else {
        0
    };
    
    format!("{} / {} ({}%)", 
            format_bytes(used), 
            format_bytes(total), 
            usage_percent)
}

pub fn format_disk_usage(used: u64, total: u64) -> String {
    let usage_percent = if total > 0 {
        (used as f64 / total as f64 * 100.0) as u8
    } else {
        0
    };
    
    let (icon, _) = format_percentage_status(100 - usage_percent);
    format!("{} {} / {} ({}% used)", 
            icon,
            format_bytes(used), 
            format_bytes(total), 
            usage_percent)
}

pub fn format_table_row(columns: &[&str], widths: &[usize]) -> String {
    let mut row = String::new();
    for (i, col) in columns.iter().enumerate() {
        if i < widths.len() {
            row.push_str(&format!("{:<width$}", col, width = widths[i]));
            if i < columns.len() - 1 {
                row.push_str(" | ");
            }
        }
    }
    row
}

pub fn format_table_separator(widths: &[usize]) -> String {
    widths.iter()
        .map(|&w| "-".repeat(w))
        .collect::<Vec<_>>()
        .join("-+-")
}

pub struct ProgressBar {
    current: u64,
    total: u64,
    width: usize,
}

impl ProgressBar {
    pub fn new(total: u64, width: usize) -> Self {
        Self {
            current: 0,
            total,
            width,
        }
    }
    
    pub fn set_current(&mut self, current: u64) {
        self.current = current;
    }
    
    pub fn format(&self) -> String {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as u8
        } else {
            0
        };
        
        let filled = if self.total > 0 {
            (self.current as f64 / self.total as f64 * self.width as f64) as usize
        } else {
            0
        };
        
        let empty = self.width.saturating_sub(filled);
        
        format!("[{}{}] {}%", 
                "█".repeat(filled).green(),
                "░".repeat(empty).dimmed(),
                percentage)
    }
}

pub fn format_service_status(running: bool, enabled: bool) -> ColoredString {
    match (running, enabled) {
        (true, true) => "🟢 Running".green(),
        (true, false) => "🟡 Running (not enabled)".yellow(),
        (false, true) => "🔴 Stopped (enabled)".red(),
        (false, false) => "⚫ Stopped".dimmed(),
    }
}

pub fn format_connection_status(connected: bool, last_seen: Option<chrono::DateTime<chrono::Utc>>) -> ColoredString {
    if connected {
        "🟢 Connected".green()
    } else {
        match last_seen {
            Some(time) => {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(time);
                if duration.num_hours() < 1 {
                    "🟡 Recently seen".yellow()
                } else if duration.num_days() < 1 {
                    "🟠 Seen today".bright_black()
                } else {
                    "🔴 Offline".red()
                }
            },
            None => "⚫ Never seen".dimmed(),
        }
    }
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub fn center_text(text: &str, width: usize) -> String {
    if text.len() >= width {
        return text.to_string();
    }
    
    let padding = width - text.len();
    let left_padding = padding / 2;
    let right_padding = padding - left_padding;
    
    format!("{}{}{}", 
            " ".repeat(left_padding),
            text,
            " ".repeat(right_padding))
}