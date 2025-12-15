use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde_json::Value;

lazy_static! {
    static ref LOCALE_CACHE: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
    static ref CURRENT_LOCALE: Mutex<String> = Mutex::new("en".to_string());
}

pub fn set_locale(locale: &str) {
    let mut current = CURRENT_LOCALE.lock().unwrap();
    *current = locale.to_string();
}

pub fn get_locale() -> String {
    CURRENT_LOCALE.lock().unwrap().clone()
}

pub fn load_locale(locale: &str, json_content: &str) {
    if let Ok(json) = serde_json::from_str(json_content) {
        let mut cache = LOCALE_CACHE.lock().unwrap();
        cache.insert(locale.to_string(), json);
    }
}

pub fn t(key: &str) -> String {
    let locale = get_locale();
    let cache = LOCALE_CACHE.lock().unwrap();
    
    // Try current locale
    if let Some(data) = cache.get(&locale) {
        if let Some(val) = get_nested(data, key) {
            return val;
        }
    }
    
    // Fallback to 'en'
    if locale != "en" {
        if let Some(data) = cache.get("en") {
            if let Some(val) = get_nested(data, key) {
                return val;
            }
        }
    }
    
    // Fallback to key itself
    key.to_string()
}

fn get_nested(value: &Value, key: &str) -> Option<String> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = value;
    
    for part in parts {
        if let Some(next) = current.get(part) {
            current = next;
        } else {
            return None;
        }
    }
    
    current.as_str().map(|s| s.to_string())
}

// Default English strings
pub fn init_defaults() {
    let en_json = r#"{
        "cli": {
            "welcome": "Welcome to Snakepit",
            "installing": "Installing package...",
            "success": "Success!",
            "error": "Error occurred"
        },
        "installer": {
            "downloading": "Downloading...",
            "verifying": "Verifying integrity...",
            "extracting": "Extracting files..."
        }
    }"#;
    load_locale("en", en_json);
}
