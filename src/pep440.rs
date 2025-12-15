use anyhow::{Result, anyhow};
use std::cmp::Ordering;
use std::fmt;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    // Regex adapted from pypa/packaging
    static ref VERSION_PATTERN: Regex = Regex::new(r"(?ix)
        \A
        v?
        (?:
            (?:(?P<epoch>[0-9]+)!)?                           # epoch
            (?P<release>[0-9]+(?:\.[0-9]+)*)                  # release segment
            (?P<pre>                                          # pre-release
                [-_\.]?
                (?P<pre_l>(a|b|c|rc|alpha|beta|pre|preview))
                [-_\.]?
                (?P<pre_n>[0-9]+)?
            )?
            (?P<post>                                         # post release
                (?:-(?P<post_n1>[0-9]+))
                |
                (?:
                    [-_\.]?
                    (?P<post_l>post|rev|r)
                    [-_\.]?
                    (?P<post_n2>[0-9]+)?
                )
            )?
            (?P<dev>                                          # dev release
                [-_\.]?
                (?P<dev_l>dev)
                [-_\.]?
                (?P<dev_n>[0-9]+)?
            )?
        )
        (?:\+(?P<local>[a-z0-9]+(?:[-_\.][a-z0-9]+)*))?       # local version
        \z
    ").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    pub epoch: u64,
    pub release: Vec<u64>,
    pub pre: Option<(String, u64)>, // (type, number)
    pub post: Option<u64>,
    pub dev: Option<u64>,
    pub local: Option<String>,
}

impl Version {
    pub fn parse(version_str: &str) -> Result<Self> {
        let caps = VERSION_PATTERN.captures(version_str.trim())
            .ok_or_else(|| anyhow!("Invalid PEP 440 version: {}", version_str))?;

        let epoch = caps.name("epoch")
            .map(|m| m.as_str().parse::<u64>())
            .transpose()?
            .unwrap_or(0);

        let release = caps.name("release")
            .map(|m| m.as_str().split('.')
                .map(|s| s.parse::<u64>())
                .collect::<Result<Vec<u64>, _>>()
            )
            .transpose()?
            .unwrap_or_default();

        let pre = if let Some(pre_l) = caps.name("pre_l") {
            let pre_type = pre_l.as_str().to_lowercase();
            let pre_n = caps.name("pre_n")
                .map(|m| m.as_str().parse::<u64>())
                .transpose()?
                .unwrap_or(0);
            
            // Normalize pre-release tags
            let normalized_type = match pre_type.as_str() {
                "alpha" => "a",
                "beta" => "b",
                "c" | "pre" | "preview" => "rc",
                _ => pre_type.as_str(),
            }.to_string();

            Some((normalized_type, pre_n))
        } else {
            None
        };

        let post = if let Some(post_n1) = caps.name("post_n1") {
            Some(post_n1.as_str().parse::<u64>()?)
        } else if let Some(_) = caps.name("post_l") {
            let post_n2 = caps.name("post_n2")
                .map(|m| m.as_str().parse::<u64>())
                .transpose()?
                .unwrap_or(0);
            Some(post_n2)
        } else {
            None
        };

        let dev = if let Some(_) = caps.name("dev_l") {
            let dev_n = caps.name("dev_n")
                .map(|m| m.as_str().parse::<u64>())
                .transpose()?
                .unwrap_or(0);
            Some(dev_n)
        } else {
            None
        };

        let local = caps.name("local").map(|m| m.as_str().to_string());

        Ok(Version {
            epoch,
            release,
            pre,
            post,
            dev,
            local,
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // 1. Epoch
        match self.epoch.cmp(&other.epoch) {
            Ordering::Equal => {},
            ord => return ord,
        }

        // 2. Release
        // Pad with zeros to match length
        let max_len = std::cmp::max(self.release.len(), other.release.len());
        for i in 0..max_len {
            let a = self.release.get(i).copied().unwrap_or(0);
            let b = other.release.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }

        // 3. Pre-release
        // Rules: No pre-release > pre-release
        match (&self.pre, &other.pre) {
            (None, None) => {},
            (Some(_), None) => return Ordering::Less,
            (None, Some(_)) => return Ordering::Greater,
            (Some((t1, n1)), Some((t2, n2))) => {
                match t1.cmp(t2) {
                    Ordering::Equal => match n1.cmp(n2) {
                        Ordering::Equal => {},
                        ord => return ord,
                    },
                    ord => return ord,
                }
            }
        }

        // 4. Post-release
        // Rules: Post-release > No post-release
        match (self.post, other.post) {
            (None, None) => {},
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (Some(n1), Some(n2)) => match n1.cmp(&n2) {
                Ordering::Equal => {},
                ord => return ord,
            }
        }

        // 5. Dev-release
        // Rules: No dev-release > dev-release
        match (self.dev, other.dev) {
            (None, None) => {},
            (Some(_), None) => return Ordering::Less,
            (None, Some(_)) => return Ordering::Greater,
            (Some(n1), Some(n2)) => match n1.cmp(&n2) {
                Ordering::Equal => {},
                ord => return ord,
            }
        }

        // 6. Local version (String comparison)
        match (&self.local, &other.local) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (Some(l1), Some(l2)) => l1.cmp(l2),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.epoch > 0 {
            write!(f, "{}!", self.epoch)?;
        }
        
        let release_str: Vec<String> = self.release.iter().map(|n| n.to_string()).collect();
        write!(f, "{}", release_str.join("."))?;

        if let Some((t, n)) = &self.pre {
            write!(f, "{}{}", t, n)?;
        }

        if let Some(n) = self.post {
            write!(f, ".post{}", n)?;
        }

        if let Some(n) = self.dev {
            write!(f, ".dev{}", n)?;
        }

        if let Some(l) = &self.local {
            write!(f, "+{}", l)?;
        }

        Ok(())
    }
}
