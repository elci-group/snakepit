//! In-tree replacement for the former `snakegg` path dependency, whose source
//! is no longer available. It preserves the small `native::*` utility surface
//! the rest of the crate actually uses, backed by `std` and a few small,
//! well-maintained crates.
//!
//! The `charmer` (LLM) and `snake_egg` (evolutionary metaphor) layers were not
//! reimplemented; every call site that used them has been removed.

pub mod native {
    /// Terminal styling helpers (ANSI colors).
    pub mod style {
        use std::fmt::Display;

        fn wrap(code: &str, text: impl Display) -> String {
            // Respect the NO_COLOR convention (https://no-color.org).
            if std::env::var_os("NO_COLOR").is_some() {
                text.to_string()
            } else {
                format!("\x1b[{code}m{text}\x1b[0m")
            }
        }

        pub fn red(text: impl Display) -> String {
            wrap("31", text)
        }
        pub fn green(text: impl Display) -> String {
            wrap("32", text)
        }
        pub fn yellow(text: impl Display) -> String {
            wrap("33", text)
        }
        pub fn blue(text: impl Display) -> String {
            wrap("34", text)
        }
        pub fn magenta(text: impl Display) -> String {
            wrap("35", text)
        }
        pub fn cyan(text: impl Display) -> String {
            wrap("36", text)
        }
        pub fn bold(text: impl Display) -> String {
            wrap("1", text)
        }
        pub fn dim(text: impl Display) -> String {
            wrap("2", text)
        }
    }

    /// Platform directories — re-exported from the `dirs` crate.
    pub mod dirs {
        pub use ::dirs::{cache_dir, config_dir, data_dir, home_dir};
    }

    /// Unique id generation (no external deps).
    pub mod id {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static COUNTER: AtomicU64 = AtomicU64::new(0);

        /// Return a process-unique, filename-safe id string.
        pub fn new() -> String {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
            format!("{:x}-{:x}-{:x}", nanos, std::process::id(), seq)
        }
    }

    /// Locate executables on `PATH`.
    pub mod which {
        use std::path::PathBuf;

        pub fn find_executable(name: &str) -> Option<PathBuf> {
            let path = std::env::var_os("PATH")?;
            for dir in std::env::split_paths(&path) {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
                #[cfg(windows)]
                {
                    let exe = dir.join(format!("{name}.exe"));
                    if exe.is_file() {
                        return Some(exe);
                    }
                }
            }
            None
        }

        pub fn has_executable(name: &str) -> bool {
            find_executable(name).is_some()
        }
    }

    /// Progress reporting — re-exported from the `indicatif` crate.
    pub mod progress {
        pub use indicatif::ProgressBar;
    }

    /// Hash helpers. `compute_hex` is MD5 (cache keys and PyPI's legacy md5
    /// digests); `compute_sha256_hex` is SHA-256.
    pub mod hash {
        use sha2::{Digest, Sha256};

        pub fn compute_hex(data: &[u8]) -> String {
            format!("{:x}", md5::compute(data))
        }

        pub fn compute_sha256_hex(data: &[u8]) -> String {
            let digest = Sha256::digest(data);
            let mut out = String::with_capacity(digest.len() * 2);
            for byte in digest.iter() {
                out.push_str(&format!("{byte:02x}"));
            }
            out
        }
    }

    /// Timestamps for logs and snapshot metadata.
    pub mod datetime {
        /// `DateTime::now().to_string()` renders the local wall-clock time.
        #[derive(Debug)]
        pub struct DateTime;

        impl DateTime {
            pub fn now() -> chrono::DateTime<chrono::Local> {
                chrono::Local::now()
            }
        }
    }

    /// Best-effort child-process reaping for the daemon tick.
    ///
    /// The original undertaker reaped zombie processes each tick. Tokio reaps
    /// the children it awaits and no detached children are spawned today, so
    /// this intentionally keeps the structure as a no-op.
    pub mod undertaker {
        #[derive(Debug, Default)]
        pub struct TheUndertaker;

        impl TheUndertaker {
            pub fn new() -> Self {
                Self
            }
            pub fn perform_rounds(&mut self) {}
        }
    }
}
