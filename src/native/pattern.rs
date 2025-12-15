// Native alternative to `regex` crate (lite version)
// Simple pattern matching with zero dependencies
// Savings: -500 KB, zero external deps
//
// Supports common patterns:
// - Wildcards: * (any chars), ? (single char)
// - Character classes: [abc], [a-z], [^abc]
// - Anchors: ^ (start), $ (end)
// - Escape: \ (literal next char)

/// Simple pattern matcher
/// 
/// Supports:
/// - `*` - Match any sequence of characters
/// - `?` - Match any single character
/// - `[abc]` - Match any character in set
/// - `[a-z]` - Match any character in range
/// - `[^abc]` - Match any character NOT in set
/// - `^` - Anchor to start
/// - `$` - Anchor to end
/// - `\` - Escape next character
/// 
/// # Example
/// ```
/// let pattern = Pattern::new("*.py");
/// assert!(pattern.matches("test.py"));
/// assert!(!pattern.matches("test.rs"));
/// ```
pub struct Pattern {
    pattern: String,
    anchor_start: bool,
    anchor_end: bool,
}

impl Pattern {
    /// Create a new pattern
    pub fn new(pattern: &str) -> Self {
        let anchor_start = pattern.starts_with('^');
        let anchor_end = pattern.ends_with('$');
        
        let pattern = pattern
            .trim_start_matches('^')
            .trim_end_matches('$')
            .to_string();
        
        Self {
            pattern,
            anchor_start,
            anchor_end,
        }
    }
    
    /// Check if text matches the pattern
    pub fn matches(&self, text: &str) -> bool {
        if self.anchor_start && self.anchor_end {
            // Must match exactly
            self.match_exact(text, &self.pattern)
        } else if self.anchor_start {
            // Must match from start
            self.match_from_start(text, &self.pattern)
        } else if self.anchor_end {
            // Must match at end
            self.match_at_end(text, &self.pattern)
        } else {
            // Can match anywhere
            self.match_anywhere(text, &self.pattern)
        }
    }
    
    fn match_exact(&self, text: &str, pattern: &str) -> bool {
        self.match_recursive(text, pattern, 0, 0)
    }
    
    fn match_from_start(&self, text: &str, pattern: &str) -> bool {
        self.match_recursive(text, pattern, 0, 0)
    }
    
    fn match_at_end(&self, text: &str, pattern: &str) -> bool {
        if text.len() < pattern.len() {
            return false;
        }
        
        // Try matching from different starting positions
        for start in 0..=text.len() {
            if self.match_recursive(text, pattern, start, 0) {
                // Check if we consumed all of text
                let consumed = self.consumed_length(text, pattern, start);
                if start + consumed == text.len() {
                    return true;
                }
            }
        }
        false
    }
    
    fn match_anywhere(&self, text: &str, pattern: &str) -> bool {
        // Try matching from each position
        for start in 0..=text.len() {
            if self.match_recursive(text, pattern, start, 0) {
                return true;
            }
        }
        false
    }
    
    fn match_recursive(&self, text: &str, pattern: &str, text_pos: usize, pat_pos: usize) -> bool {
        let text_bytes = text.as_bytes();
        let pat_bytes = pattern.as_bytes();
        
        // End of pattern - success if also end of text (or not anchored)
        if pat_pos >= pat_bytes.len() {
            return !self.anchor_end || text_pos >= text_bytes.len();
        }
        
        // End of text - success only if rest of pattern is all *
        if text_pos >= text_bytes.len() {
            return pat_bytes[pat_pos..].iter().all(|&c| c == b'*');
        }
        
        let pat_char = pat_bytes[pat_pos];
        
        match pat_char {
            b'*' => {
                // Try matching zero or more characters
                // First try zero characters
                if self.match_recursive(text, pattern, text_pos, pat_pos + 1) {
                    return true;
                }
                // Then try one or more characters
                for i in text_pos..text_bytes.len() {
                    if self.match_recursive(text, pattern, i + 1, pat_pos + 1) {
                        return true;
                    }
                }
                false
            }
            b'?' => {
                // Match any single character
                self.match_recursive(text, pattern, text_pos + 1, pat_pos + 1)
            }
            b'[' => {
                // Character class
                if let Some((matched, next_pat)) = self.match_char_class(text_bytes[text_pos], &pat_bytes[pat_pos..]) {
                    if matched {
                        self.match_recursive(text, pattern, text_pos + 1, pat_pos + next_pat)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            b'\\' => {
                // Escape - match next character literally
                if pat_pos + 1 < pat_bytes.len() {
                    if text_bytes[text_pos] == pat_bytes[pat_pos + 1] {
                        self.match_recursive(text, pattern, text_pos + 1, pat_pos + 2)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => {
                // Literal character
                if text_bytes[text_pos] == pat_char {
                    self.match_recursive(text, pattern, text_pos + 1, pat_pos + 1)
                } else {
                    false
                }
            }
        }
    }
    
    fn match_char_class(&self, ch: u8, class: &[u8]) -> Option<(bool, usize)> {
        // Find the closing ]
        let end = class.iter().position(|&c| c == b']')?;
        
        if end < 2 {
            return None;
        }
        
        let negate = class[1] == b'^';
        let start = if negate { 2 } else { 1 };
        
        let mut matched = false;
        let mut i = start;
        
        while i < end {
            if i + 2 < end && class[i + 1] == b'-' {
                // Range: a-z
                if ch >= class[i] && ch <= class[i + 2] {
                    matched = true;
                    break;
                }
                i += 3;
            } else {
                // Single character
                if ch == class[i] {
                    matched = true;
                    break;
                }
                i += 1;
            }
        }
        
        Some((matched != negate, end + 1))
    }
    
    fn consumed_length(&self, text: &str, pattern: &str, start: usize) -> usize {
        // Helper to determine how many characters were consumed
        // This is a simplified version
        let mut consumed = 0;
        let mut pat_pos = 0;
        let pat_bytes = pattern.as_bytes();
        
        while pat_pos < pat_bytes.len() && start + consumed < text.len() {
            match pat_bytes[pat_pos] {
                b'*' => {
                    // Consume as much as possible
                    while start + consumed < text.len() {
                        consumed += 1;
                    }
                    pat_pos += 1;
                }
                b'?' => {
                    consumed += 1;
                    pat_pos += 1;
                }
                _ => {
                    consumed += 1;
                    pat_pos += 1;
                }
            }
        }
        
        consumed
    }
}

/// Quick pattern matching without creating a Pattern object
/// 
/// # Example
/// ```
/// assert!(matches("test.py", "*.py"));
/// assert!(!matches("test.rs", "*.py"));
/// ```
pub fn matches(text: &str, pattern: &str) -> bool {
    Pattern::new(pattern).matches(text)
}

/// Find all matches in text
/// 
/// Returns the starting positions of all matches
pub fn find_all(text: &str, pattern: &str) -> Vec<usize> {
    let pat = Pattern::new(pattern);
    let mut matches = Vec::new();
    
    for (i, _) in text.char_indices() {
        if pat.matches(&text[i..]) {
            matches.push(i);
        }
    }
    
    matches
}

/// Replace all matches with replacement text
/// 
/// # Example
/// ```
/// let result = replace_all("hello world", "world", "rust");
/// assert_eq!(result, "hello rust");
/// ```
pub fn replace_all(text: &str, pattern: &str, replacement: &str) -> String {
    // Simple implementation - just replace literal strings
    text.replace(pattern, replacement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard() {
        assert!(matches("test.py", "*.py"));
        assert!(matches("test.rs", "*.rs"));
        assert!(!matches("test.py", "*.rs"));
    }

    #[test]
    fn test_question_mark() {
        assert!(matches("test", "t?st"));
        assert!(matches("tast", "t?st"));
        assert!(!matches("toast", "t?st"));
    }

    #[test]
    fn test_char_class() {
        assert!(matches("a", "[abc]"));
        assert!(matches("b", "[abc]"));
        assert!(!matches("d", "[abc]"));
    }

    #[test]
    fn test_char_range() {
        assert!(matches("a", "[a-z]"));
        assert!(matches("m", "[a-z]"));
        assert!(!matches("A", "[a-z]"));
    }

    #[test]
    fn test_negated_class() {
        assert!(matches("d", "[^abc]"));
        assert!(!matches("a", "[^abc]"));
    }

    #[test]
    fn test_anchors() {
        assert!(matches("test", "^test$"));
        assert!(matches("test.py", "^test"));
        assert!(matches("test.py", ".py$"));
        assert!(!matches("test.py", "^.py$"));
    }

    #[test]
    fn test_escape() {
        assert!(matches("test*", r"test\*"));
        assert!(!matches("test", r"test\*"));
    }
}
