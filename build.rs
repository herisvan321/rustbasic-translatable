use std::fs;
use std::path::PathBuf;
use std::env;

fn main() {
    // Only run if not compiling for docs.rs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    // Determine parent project root directory
    let project_root = match env::var("PWD") {
        Ok(pwd) => PathBuf::from(pwd),
        Err(_) => match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return,
        },
    };

    // Ensure it's a Cargo project
    if !project_root.join("Cargo.toml").exists() {
        return;
    }

    // Avoid scaffolding if we are debugging/building the library itself
    if project_root.join("src/middleware.rs").exists() && project_root.join("src/translator.rs").exists() {
        return;
    }

    // 1. Create lang directory
    let lang_dir = project_root.join("lang");
    fs::create_dir_all(&lang_dir).ok();

    // 2. Create Indonesian defaults if not exists
    let id_json_path = lang_dir.join("id.json");
    if !id_json_path.exists() {
        let id_template = r#"{
  "welcome": "Selamat datang di aplikasi kami!",
  "about": "Tentang Kami",
  "welcome_user": "Selamat datang kembali, {name}!",
  "auth": {
    "failed": "Kredensial yang dimasukkan tidak cocok dengan catatan kami.",
    "throttle": "Terlalu banyak percobaan masuk. Silakan coba lagi dalam {seconds} detik."
  }
}
"#;
        fs::write(id_json_path, id_template).ok();
    }

    // 3. Create English defaults if not exists
    let en_json_path = lang_dir.join("en.json");
    if !en_json_path.exists() {
        let en_template = r#"{
  "welcome": "Welcome to our application!",
  "about": "About Us",
  "welcome_user": "Welcome back, {name}!",
  "auth": {
    "failed": "These credentials do not match our records.",
    "throttle": "Too many login attempts. Please try again in {seconds} seconds."
  }
}
"#;
        fs::write(en_json_path, en_template).ok();
    }
}
