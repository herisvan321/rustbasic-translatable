use axum::{
    body::Body,
    http::{Request, HeaderMap},
    middleware::Next,
    response::Response,
};
use crate::translator::TRANSLATOR;

// Define task-local active request locale storage
tokio::task_local! {
    pub static CURRENT_LOCALE: String;
}

/// Axum Middleware to extract client's preferred language and bind it to the request scope
pub async fn translatable_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let extracted_locale = detect_locale(&req);

    // Bind locale to task-local context so it is available thread-safely throughout the request lifetime
    CURRENT_LOCALE.scope(extracted_locale, next.run(req)).await
}

/// Retrieve the active request locale, falling back to default locale if not bound
pub fn get_locale() -> String {
    CURRENT_LOCALE.try_with(|l| l.clone()).unwrap_or_else(|_| TRANSLATOR.get_default_locale())
}

/// Detect language from request query, cookies, session, or Accept-Language header
fn detect_locale(req: &Request<Body>) -> String {
    let headers = req.headers();

    // 1. Cek Query Parameter: ?lang=en atau ?locale=en
    if let Some(q_locale) = extract_query(req.uri()) {
        return q_locale;
    }

    // 2. Cek Axum Session jika tersedia
    let session_opt = req.extensions().get::<rustbasic_core::axum_session::Session<rustbasic_core::session_manager::RustBasicSessionStore>>();
    if let Some(s_locale) = session_opt.and_then(|session| session.get::<String>("locale")) {
        let trimmed = s_locale.trim().to_lowercase();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    // 3. Cek Cookie: lang=en atau locale=en
    if let Some(c_locale) = extract_cookie(headers) {
        return c_locale;
    }

    // 4. Cek Header Accept-Language: id-ID,id;q=0.9,en-US;q=0.8
    if let Some(h_locale) = extract_accept_language(headers) {
        return h_locale;
    }

    // 5. Fallback ke default global translator
    TRANSLATOR.get_default_locale()
}

/// Helper to parse locale from URI Query String
fn extract_query(uri: &axum::http::Uri) -> Option<String> {
    let query = uri.query()?;
    for pair in query.split('&') {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 && (parts[0] == "lang" || parts[0] == "locale") {
            let val = parts[1].trim();
            if !val.is_empty() {
                return Some(val.to_lowercase());
            }
        }
    }
    None
}

/// Helper to parse locale from Request Cookies
fn extract_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for cookie in cookie_header.split(';') {
        let parts: Vec<&str> = cookie.splitn(2, '=').collect();
        if parts.len() == 2 {
            let name = parts[0].trim();
            if name == "lang" || name == "locale" {
                let val = parts[1].trim();
                if !val.is_empty() {
                    return Some(val.to_lowercase());
                }
            }
        }
    }
    None
}

/// Helper to parse locale from Accept-Language Header
fn extract_accept_language(headers: &HeaderMap) -> Option<String> {
    let accept_lang = headers.get(axum::http::header::ACCEPT_LANGUAGE)?.to_str().ok()?;
    // Contoh: "id-ID,id;q=0.9,en-US;q=0.8" -> Ambil bagian pertama sebelum koma -> "id-ID"
    let first_part = accept_lang.split(',').next()?;
    // Ambil bagian bahasa utama sebelum tanda minus -> "id"
    let lang_code = first_part.split(';').next()?.trim();
    let primary_code = lang_code.split('-').next()?.trim();
    if primary_code.len() == 2 {
        Some(primary_code.to_lowercase())
    } else {
        None
    }
}
