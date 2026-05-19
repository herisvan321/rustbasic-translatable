use std::collections::HashMap;
use std::sync::RwLock;
use std::fs;
use std::path::Path;
use once_cell::sync::Lazy;

/// Core Translator struct containing thread-safe mappings of locales to flat key-value pairs
pub struct Translator {
    translations: RwLock<HashMap<String, HashMap<String, String>>>,
    default_locale: RwLock<String>,
}

/// Global, thread-safe instance of the Translator
pub static TRANSLATOR: Lazy<Translator> = Lazy::new(|| Translator {
    translations: RwLock::new(HashMap::new()),
    default_locale: RwLock::new("id".to_string()),
});

impl Translator {
    /// Initialize and load all JSON translation files from the specified folder
    pub fn init<P: AsRef<Path>>(&self, lang_dir: P) -> Result<(), String> {
        let path = lang_dir.as_ref();
        if !path.exists() || !path.is_dir() {
            return Err(format!("Direktori bahasa tidak ditemukan atau bukan folder: {:?}", path));
        }

        let mut all_translations = HashMap::new();

        let entries = fs::read_dir(path)
            .map_err(|e| format!("Gagal membaca direktori bahasa: {}", e))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let file_path = entry.path();
            if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                let locale = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| format!("Nama file tidak valid: {:?}", file_path))?
                    .to_string();

                let content = fs::read_to_string(&file_path)
                    .map_err(|e| format!("Gagal membaca file {:?}: {}", file_path, e))?;

                let json_value: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| format!("Gagal mem-parse JSON di {:?}: {}", file_path, e))?;

                let mut flat_map = HashMap::new();
                flatten_json("", &json_value, &mut flat_map);

                all_translations.insert(locale, flat_map);
            }
        }

        let mut write_lock = self.translations.write().unwrap();
        *write_lock = all_translations;

        Ok(())
    }

    /// Set default fallback locale
    pub fn set_default_locale(&self, locale: &str) {
        let mut default_lock = self.default_locale.write().unwrap();
        *default_lock = locale.to_string();
    }

    /// Get current default fallback locale
    pub fn get_default_locale(&self) -> String {
        let default_lock = self.default_locale.read().unwrap();
        default_lock.clone()
    }

    /// Translate a key for the given locale with fallbacks
    pub fn trans(&self, key: &str, locale: &str) -> String {
        let translations = self.translations.read().unwrap();

        // 1. Coba cari di bahasa yang di-request
        if let Some(val) = translations.get(locale).and_then(|m| m.get(key)) {
            return val.clone();
        }

        // 2. Fallback ke bahasa default
        let default_loc = self.get_default_locale();
        if locale != default_loc {
            let opt = translations.get(&default_loc).and_then(|m| m.get(key));
            if let Some(val) = opt {
                return val.clone();
            }
        }

        // 3. Fallback terakhir: kembalikan key itu sendiri
        key.to_string()
    }

    /// Translate a key with dynamic placeholder parameter replacement
    pub fn trans_with(&self, key: &str, locale: &str, params: &[(&str, &str)]) -> String {
        let mut translated = self.trans(key, locale);
        for &(param, val) in params {
            let placeholder = format!("{{{}}}", param);
            translated = translated.replace(&placeholder, val);
        }
        translated
    }
}

/// Recursive helper to flatten nested JSON structures into a dot-separated flat map
fn flatten_json(prefix: &str, value: &serde_json::Value, flat_map: &mut HashMap<String, String>) {
    match value {
        serde_json::Value::Object(obj) => {
            for (k, v) in obj {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(&new_prefix, v, flat_map);
            }
        }
        serde_json::Value::String(s) => {
            flat_map.insert(prefix.to_string(), s.clone());
        }
        serde_json::Value::Number(n) => {
            flat_map.insert(prefix.to_string(), n.to_string());
        }
        serde_json::Value::Bool(b) => {
            flat_map.insert(prefix.to_string(), b.to_string());
        }
        serde_json::Value::Null => {
            flat_map.insert(prefix.to_string(), String::new());
        }
        serde_json::Value::Array(arr) => {
            flat_map.insert(prefix.to_string(), serde_json::Value::Array(arr.clone()).to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_flatten_json() {
        let json_str = r#"{
            "welcome": "Selamat datang",
            "nested": {
                "level1": {
                    "level2": "Sukses"
                }
            }
        }"#;

        let val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let mut map = HashMap::new();
        flatten_json("", &val, &mut map);

        assert_eq!(map.get("welcome").unwrap(), "Selamat datang");
        assert_eq!(map.get("nested.level1.level2").unwrap(), "Sukses");
    }

    #[test]
    fn test_translator_flow() {
        // Create temporary lang directory
        let temp_dir = std::env::temp_dir().join("lang_test_dir");
        fs::create_dir_all(&temp_dir).ok();

        // Write id.json
        let mut id_file = File::create(temp_dir.join("id.json")).unwrap();
        id_file.write_all(r#"{
            "welcome": "Selamat datang!",
            "greet": "Halo, {name}!",
            "auth": {
                "failed": "Gagal masuk."
            }
        }"#.as_bytes()).unwrap();

        // Write en.json
        let mut en_file = File::create(temp_dir.join("en.json")).unwrap();
        en_file.write_all(r#"{
            "welcome": "Welcome!",
            "greet": "Hello, {name}!"
        }"#.as_bytes()).unwrap();

        // Initialize translator
        let translator = Translator {
            translations: RwLock::new(HashMap::new()),
            default_locale: RwLock::new("id".to_string()),
        };
        translator.init(&temp_dir).unwrap();

        // Test normal translations
        assert_eq!(translator.trans("welcome", "id"), "Selamat datang!");
        assert_eq!(translator.trans("welcome", "en"), "Welcome!");

        // Test nested translation
        assert_eq!(translator.trans("auth.failed", "id"), "Gagal masuk.");

        // Test fallback to default (id)
        assert_eq!(translator.trans("auth.failed", "en"), "Gagal masuk.");

        // Test missing key fallback to key itself
        assert_eq!(translator.trans("missing.key", "en"), "missing.key");

        // Test placeholder replacement
        assert_eq!(translator.trans_with("greet", "id", &[("name", "Hendra")]), "Halo, Hendra!");
        assert_eq!(translator.trans_with("greet", "en", &[("name", "John")]), "Hello, John!");

        // Clean up
        fs::remove_dir_all(&temp_dir).ok();
    }
}
