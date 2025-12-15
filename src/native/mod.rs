// Native lightweight alternatives to external dependencies
// Zero external dependencies, minimal binary size impact

pub mod which;
pub mod id;
pub mod pattern;
pub mod datetime;
pub mod watch;
pub mod style;
pub mod progress;
pub mod dirs;
pub mod hash;

// Re-export commonly used items

// Tier C replacements
pub use which::{find_executable, has_executable};
pub use id::{generate_id, generate_short_id, generate_uuid_v4, generate_sortable_id};
pub use pattern::{Pattern, matches as pattern_matches};
pub use datetime::{DateTime, format_duration};

// Tier B replacements
pub use watch::{Watcher, WatchEvent, watch_file};
pub use style::{StyledText, Color, red, green, yellow, blue, cyan, magenta, bold, dim};
pub use progress::{ProgressBar, Spinner, MultiProgress};
pub use dirs::{home_dir, cache_dir, config_dir, data_dir, temp_dir};
pub use hash::{compute as md5_compute, format_hash as md5_format, compute_hex, compute_sha256_hex};
pub mod i18n;
pub mod ollama;
pub mod hardware;
pub mod undertaker;
