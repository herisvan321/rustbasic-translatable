pub mod translator;
pub mod middleware;

// Re-export the Axum middleware for easy integration in parent routers
pub use middleware::translatable_middleware;

/// Translate a key using the active request-scoped locale automatically
pub fn trans(key: &str) -> String {
    let locale = middleware::get_locale();
    translator::TRANSLATOR.trans(key, &locale)
}

/// Translate a key with dynamic placeholder variables using the active request-scoped locale
pub fn trans_with(key: &str, params: &[(&str, &str)]) -> String {
    let locale = middleware::get_locale();
    translator::TRANSLATOR.trans_with(key, &locale, params)
}

/// Standard global translation shortcut function, identical to trans
pub fn __(key: &str) -> String {
    trans(key)
}

/// Initialize and load all JSON translation files from the specified folder
pub fn init<P: AsRef<std::path::Path>>(lang_dir: P) -> Result<(), String> {
    translator::TRANSLATOR.init(lang_dir)
}

/// Set default fallback language locale (e.g. "id")
pub fn set_default_locale(locale: &str) {
    translator::TRANSLATOR.set_default_locale(locale);
}

/// Get current default fallback language locale
pub fn get_default_locale() -> String {
    translator::TRANSLATOR.get_default_locale()
}
