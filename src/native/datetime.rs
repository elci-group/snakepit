// Native alternative to `chrono` crate
// Simple date/time handling with zero dependencies
// Savings: -200 KB, zero external deps

use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Simple DateTime representation
/// 
/// Stores time as milliseconds since UNIX_EPOCH
/// Provides common formatting and parsing operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    millis: u64,
}

impl DateTime {
    /// Get current time
    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self { millis }
    }
    
    /// Create from UNIX timestamp (seconds)
    pub fn from_timestamp(secs: u64) -> Self {
        Self { millis: secs * 1000 }
    }
    
    /// Create from UNIX timestamp (milliseconds)
    pub fn from_timestamp_millis(millis: u64) -> Self {
        Self { millis }
    }
    
    /// Get UNIX timestamp (seconds)
    pub fn timestamp(&self) -> u64 {
        self.millis / 1000
    }
    
    /// Get UNIX timestamp (milliseconds)
    pub fn timestamp_millis(&self) -> u64 {
        self.millis
    }
    
    /// Format as ISO 8601 string
    /// 
    /// Format: YYYY-MM-DDTHH:MM:SS.sssZ
    /// Example: 2024-12-15T10:30:45.123Z
    pub fn to_iso8601(&self) -> String {
        let (year, month, day, hour, minute, second, millis) = self.to_components();
        
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            year, month, day, hour, minute, second, millis
        )
    }
    
    /// Format as RFC 3339 string (same as ISO 8601 for UTC)
    pub fn to_rfc3339(&self) -> String {
        self.to_iso8601()
    }
    
    /// Format as human-readable string
    /// 
    /// Format: YYYY-MM-DD HH:MM:SS
    /// Example: 2024-12-15 10:30:45
    pub fn to_string(&self) -> String {
        let (year, month, day, hour, minute, second, _) = self.to_components();
        
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    }
    
    /// Format as date only
    /// 
    /// Format: YYYY-MM-DD
    /// Example: 2024-12-15
    pub fn to_date_string(&self) -> String {
        let (year, month, day, _, _, _, _) = self.to_components();
        format!("{:04}-{:02}-{:02}", year, month, day)
    }
    
    /// Format as time only
    /// 
    /// Format: HH:MM:SS
    /// Example: 10:30:45
    pub fn to_time_string(&self) -> String {
        let (_, _, _, hour, minute, second, _) = self.to_components();
        format!("{:02}:{:02}:{:02}", hour, minute, second)
    }
    
    /// Parse from ISO 8601 string
    /// 
    /// Supports: YYYY-MM-DDTHH:MM:SS.sssZ
    pub fn parse_iso8601(s: &str) -> Option<Self> {
        // Simple parser for ISO 8601
        let s = s.trim_end_matches('Z');
        let parts: Vec<&str> = s.split('T').collect();
        
        if parts.len() != 2 {
            return None;
        }
        
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        if date_parts.len() != 3 {
            return None;
        }
        
        let time_parts: Vec<&str> = parts[1].split(':').collect();
        if time_parts.len() != 3 {
            return None;
        }
        
        let year: u32 = date_parts[0].parse().ok()?;
        let month: u32 = date_parts[1].parse().ok()?;
        let day: u32 = date_parts[2].parse().ok()?;
        
        let hour: u32 = time_parts[0].parse().ok()?;
        let minute: u32 = time_parts[1].parse().ok()?;
        
        // Handle seconds with optional milliseconds
        let sec_parts: Vec<&str> = time_parts[2].split('.').collect();
        let second: u32 = sec_parts[0].parse().ok()?;
        let millis: u32 = if sec_parts.len() > 1 {
            sec_parts[1].parse().ok()?
        } else {
            0
        };
        
        Self::from_components(year, month, day, hour, minute, second, millis)
    }
    
    /// Create from date/time components
    pub fn from_components(
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        millis: u32,
    ) -> Option<Self> {
        // Calculate days since epoch
        let days = days_since_epoch(year, month, day)?;
        
        let total_millis = 
            days as u64 * 86400 * 1000 +
            hour as u64 * 3600 * 1000 +
            minute as u64 * 60 * 1000 +
            second as u64 * 1000 +
            millis as u64;
        
        Some(Self { millis: total_millis })
    }
    
    /// Convert to date/time components
    /// 
    /// Returns: (year, month, day, hour, minute, second, millis)
    pub fn to_components(&self) -> (u32, u32, u32, u32, u32, u32, u32) {
        let total_seconds = self.millis / 1000;
        let millis = (self.millis % 1000) as u32;
        
        let days = total_seconds / 86400;
        let remaining = total_seconds % 86400;
        
        let hour = (remaining / 3600) as u32;
        let minute = ((remaining % 3600) / 60) as u32;
        let second = (remaining % 60) as u32;
        
        let (year, month, day) = days_to_date(days);
        
        (year, month, day, hour, minute, second, millis)
    }
    
    /// Add duration
    pub fn add(&self, duration: Duration) -> Self {
        Self {
            millis: self.millis + duration.as_millis() as u64,
        }
    }
    
    /// Subtract duration
    pub fn sub(&self, duration: Duration) -> Self {
        Self {
            millis: self.millis.saturating_sub(duration.as_millis() as u64),
        }
    }
    
    /// Duration since another DateTime
    pub fn duration_since(&self, other: &DateTime) -> Duration {
        Duration::from_millis(self.millis.saturating_sub(other.millis))
    }
    
    /// Elapsed time since this DateTime
    pub fn elapsed(&self) -> Duration {
        DateTime::now().duration_since(self)
    }
}

/// Calculate days since UNIX epoch for a given date
fn days_since_epoch(year: u32, month: u32, day: u32) -> Option<u32> {
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    
    let mut days = 0u32;
    
    // Add days for complete years since 1970
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    
    // Add days for complete months in current year
    for m in 1..month {
        days += days_in_month(year, m);
    }
    
    // Add remaining days
    days += day - 1;
    
    Some(days)
}

/// Convert days since epoch to date
fn days_to_date(mut days: u64) -> (u32, u32, u32) {
    let mut year = 1970u32;
    
    // Find year
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year as u64 {
            break;
        }
        days -= days_in_year as u64;
        year += 1;
    }
    
    // Find month
    let mut month = 1u32;
    while month <= 12 {
        let days_in_month = days_in_month(year, month) as u64;
        if days < days_in_month {
            break;
        }
        days -= days_in_month;
        month += 1;
    }
    
    let day = (days + 1) as u32;
    
    (year, month, day)
}

/// Check if year is a leap year
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get number of days in a month
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

/// Format duration as human-readable string
/// 
/// Examples:
/// - "2d 3h 45m"
/// - "1h 30m 15s"
/// - "45s"
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    
    let mut parts = Vec::new();
    
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }
    
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let dt = DateTime::now();
        assert!(dt.timestamp() > 1700000000); // After 2023
    }

    #[test]
    fn test_from_timestamp() {
        let dt = DateTime::from_timestamp(1700000000);
        assert_eq!(dt.timestamp(), 1700000000);
    }

    #[test]
    fn test_iso8601() {
        let dt = DateTime::from_timestamp(1700000000);
        let iso = dt.to_iso8601();
        assert!(iso.contains("T"));
        assert!(iso.ends_with("Z"));
    }

    #[test]
    fn test_parse_iso8601() {
        let iso = "2024-12-15T10:30:45.123Z";
        let dt = DateTime::parse_iso8601(iso).unwrap();
        let parsed_iso = dt.to_iso8601();
        assert_eq!(iso, parsed_iso);
    }

    #[test]
    fn test_components() {
        let dt = DateTime::from_components(2024, 12, 15, 10, 30, 45, 0).unwrap();
        let (y, m, d, h, min, s, _) = dt.to_components();
        assert_eq!((y, m, d, h, min, s), (2024, 12, 15, 10, 30, 45));
    }

    #[test]
    fn test_duration() {
        let dt1 = DateTime::now();
        let dt2 = dt1.add(Duration::from_secs(3600));
        let diff = dt2.duration_since(&dt1);
        assert_eq!(diff.as_secs(), 3600);
    }

    #[test]
    fn test_format_duration() {
        let dur = Duration::from_secs(3665); // 1h 1m 5s
        let formatted = format_duration(dur);
        assert!(formatted.contains("1h"));
        assert!(formatted.contains("1m"));
        assert!(formatted.contains("5s"));
    }
}
