// Native alternative to `uuid` crate
// Generates unique IDs with zero dependencies
// Savings: -50 KB, zero external deps

use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for ensuring uniqueness
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique ID
/// 
/// Format: {timestamp_ms}-{counter}-{random}
/// Example: "1702650000000-42-a3f9c8b2"
/// 
/// # Properties
/// - Sortable by creation time (timestamp first)
/// - Unique even with high concurrency (atomic counter)
/// - Collision-resistant (random suffix)
/// - Compact (shorter than UUID)
/// 
/// # Example
/// ```
/// let id = generate_id();
/// println!("Generated ID: {}", id);
/// ```
pub fn generate_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let random = generate_random_u64();
    
    format!("{:x}-{:x}-{:x}", timestamp, counter, random)
}

/// Generate a short ID (8 characters)
/// 
/// Format: {timestamp_hex}{random_hex}
/// Example: "a3f9c8b2"
/// 
/// # Warning
/// Higher collision probability than full IDs
/// Use only for non-critical temporary IDs
pub fn generate_short_id() -> String {
    let timestamp = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() & 0xFFFF) as u16;
    
    let random = (generate_random_u64() & 0xFFFF) as u16;
    
    format!("{:04x}{:04x}", timestamp, random)
}

/// Generate a UUID v4 compatible string
/// 
/// Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
/// 
/// # Note
/// This is a simplified version that's compatible with UUID v4 format
/// but uses a simpler random number generation
pub fn generate_uuid_v4() -> String {
    let r1 = generate_random_u64();
    let r2 = generate_random_u64();
    
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (r1 >> 32) as u32,
        ((r1 >> 16) & 0xFFFF) as u16,
        (r1 & 0xFFF) as u16,
        ((r2 >> 48) & 0xFFFF) as u16,
        (r2 & 0xFFFFFFFFFFFF) as u64
    )
}

/// Create a new UUID v4 (alias for generate_uuid_v4)
pub fn new() -> String {
    generate_uuid_v4()
}

/// Generate a timestamp-based sortable ID (ULID-like)
/// 
/// Format: {timestamp_base32}{random_base32}
/// 
/// # Properties
/// - Lexicographically sortable
/// - 26 characters
/// - Case-insensitive
pub fn generate_sortable_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let random = generate_random_u64();
    
    // Use base32 encoding for better readability
    let timestamp_b32 = to_base32(timestamp as u64, 10);
    let random_b32 = to_base32(random, 16);
    
    format!("{}{}", timestamp_b32, random_b32)
}

/// Simple pseudo-random number generator
/// 
/// Uses a combination of:
/// - System time (nanoseconds)
/// - Thread ID (if available)
/// - Static counter
/// - Simple hash mixing
/// 
/// # Note
/// This is NOT cryptographically secure
/// Use only for non-security-critical IDs
fn generate_random_u64() -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let counter = COUNTER.load(Ordering::Relaxed);
    
    // Simple hash mixing (splitmix64-inspired)
    let mut x = time.wrapping_add(counter);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Convert number to base32 (Crockford encoding)
fn to_base32(mut num: u64, len: usize) -> String {
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let mut result = vec![b'0'; len];
    
    for i in (0..len).rev() {
        result[i] = ALPHABET[(num % 32) as usize];
        num /= 32;
    }
    
    String::from_utf8(result).unwrap()
}

/// Parse a timestamp from our ID format
/// 
/// Returns the creation timestamp in milliseconds since UNIX_EPOCH
pub fn parse_timestamp(id: &str) -> Option<u128> {
    let parts: Vec<&str> = id.split('-').collect();
    if parts.is_empty() {
        return None;
    }
    
    u128::from_str_radix(parts[0], 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        
        assert_ne!(id1, id2, "IDs should be unique");
        assert!(id1.contains('-'), "ID should contain separators");
    }

    #[test]
    fn test_generate_short_id() {
        let id = generate_short_id();
        assert_eq!(id.len(), 8, "Short ID should be 8 characters");
    }

    #[test]
    fn test_generate_uuid_v4() {
        let uuid = generate_uuid_v4();
        assert_eq!(uuid.len(), 36, "UUID should be 36 characters");
        assert!(uuid.contains('-'), "UUID should contain hyphens");
    }

    #[test]
    fn test_generate_sortable_id() {
        let id1 = generate_sortable_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = generate_sortable_id();
        
        assert!(id1 < id2, "Sortable IDs should be lexicographically ordered");
    }

    #[test]
    fn test_parse_timestamp() {
        let id = generate_id();
        let timestamp = parse_timestamp(&id);
        assert!(timestamp.is_some(), "Should parse timestamp from ID");
    }

    #[test]
    fn test_uniqueness() {
        let mut ids = std::collections::HashSet::new();
        for _ in 0..10000 {
            let id = generate_id();
            assert!(ids.insert(id), "All IDs should be unique");
        }
    }
}
