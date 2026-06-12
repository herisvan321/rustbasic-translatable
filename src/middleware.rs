use rustbasic_core::requests::Request;
use rustbasic_core::middleware::Next;
use rustbasic_core::router::Response;
use crate::translator::TRANSLATOR;

// Define task-local active request locale storage
rustbasic_core::tokio::task_local! {
    pub static CURRENT_LOCALE: String;
}

/// RustBasic Middleware to extract client's preferred language and bind it to the request scope
pub async fn translatable_middleware(
    req: Request,
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
fn detect_locale(req: &Request) -> String {
    // 1. Cek Query/Input Parameter: ?lang=en atau ?locale=en
    if let Some(q_locale) = req.input_as_str("lang").or_else(|| req.input_as_str("locale")) {
        let trimmed = q_locale.trim().to_lowercase();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    // 2. Cek Session jika tersedia
    if let Some(s_locale) = req.session.get::<String>("locale") {
        let trimmed = s_locale.trim().to_lowercase();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    // 3. Cek Cookie: lang=en atau locale=en
    if let Some(cookie_header) = req.headers.get("cookie") {
        for cookie in cookie_header.split(';') {
            let parts: Vec<&str> = cookie.splitn(2, '=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim();
                if name == "lang" || name == "locale" {
                    let val = parts[1].trim();
                    if !val.is_empty() {
                        return val.to_lowercase();
                    }
                }
            }
        }
    }

    // 4. Cek Header Accept-Language: id-ID,id;q=0.9,en-US;q=0.8
    if let Some(accept_lang) = req.headers.get("accept-language")
        && let Some(first_part) = accept_lang.split(',').next()
        && let Some(lang_code) = first_part.split(';').next() {
        let primary_code = lang_code.trim().split('-').next().unwrap_or("").trim();
        if primary_code.len() == 2 {
            return primary_code.to_lowercase();
        }
    }

    // 5. Fallback ke default global translator
    TRANSLATOR.get_default_locale()
}
