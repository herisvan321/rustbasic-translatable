# 🌐 RustBasic Translatable

`rustbasic-translatable` adalah paket lokalisasi multi-bahasa resmi (i18n) untuk **RustBasic Framework**. Paket ini menyediakan manajemen bahasa yang aman, cepat (`O(1)`), dan berbasis file JSON yang secara otomatis mengonfigurasi dirinya sendiri (*zero-config scaffolding*) saat dipasang.

Dengan memanfaatkan kemampuan canggih `tokio::task_local!`, paket ini mampu mengikat bahasa aktif ke setiap request thread secara asinkron. Anda dapat memanggil fungsi terjemahan `__("kunci")` langsung dari mana saja di seluruh basis kode Anda tanpa harus mengoper argumen bahasa secara manual di setiap baris fungsi.

---

## ✨ Fitur Utama

- 📂 **Zero-Config Scaffolding**: Secara otomatis membuat folder `lang/` di root proyek induk Anda lengkap dengan template bahasa bawaan (`id.json` dan `en.json`) saat proyek dikompilasi pertama kali.
- ⚡ **Pencarian Kilat $O(1)$**: Mengurai file JSON bersarang (*nested JSON*) secara rekursif pada startup dan meratakannya (*flattening*) ke memori dalam bentuk *dot-separated keys* (contoh: `auth.failed`) agar lookup saat runtime berjalan super cepat.
- 🧵 **Request-Scoped Locale**: Mengikat preferensi bahasa aktif ke *connection task* menggunakan `tokio::task_local!`, memungkinkan pemanggilan fungsi `trans` atau `__` secara global.
- 🎯 **Deteksi Bahasa Pintar (RustBasic Middleware)**: Mendeteksi preferensi bahasa pengguna secara otomatis berdasarkan prioritas berikut:
  1. **Query Parameter**: `?lang=en` atau `?locale=en`
  2. **Session Rust**: Membaca kunci `"locale"` di session store aplikasi Anda.
  3. **Request Cookie**: Cookie `lang` atau `locale`.
  4. **Header Browser**: Menganalisis header `Accept-Language` browser pengguna secara otomatis.
- 🔄 **Placeholder Dinamis**: Mengganti placeholder variabel seperti `{name}` dengan nilai dinamis saat runtime secara instan.
- 🛡️ **Sistem Fallback Kokoh**: Jika terjemahan tidak ditemukan pada bahasa aktif, sistem akan otomatis beralih ke bahasa default (`id`), dan jika masih tidak ditemukan, akan menampilkan kunci itu sendiri sebagai cadangan terakhir.

---

## 📦 Pemasangan

Cukup tambahkan `rustbasic-translatable` ke dalam blok dependensi `Cargo.toml` pada proyek induk Anda:

```toml
[dependencies]
rustbasic-core = { path = "../rustbasic-core", version = "0.1" }
rustbasic-translatable = { path = "../rustbasic-translatable", version = "0.0" }
```

Saat Anda menjalankan perintah `cargo build` atau `cargo check` berikutnya, paket ini akan langsung memicu pembuatan otomatis folder bahasa Anda!

---

## ⚙️ Cara Kerja Scaffolding Otomatis

Saat dipasang, build script (`build.rs`) mendeteksi root proyek induk Anda dan secara otomatis membuat struktur berkas berikut:

```text
proyek-anda/
├── lang/
│   ├── id.json       # Template Terjemahan Bahasa Indonesia
│   └── en.json       # Template Terjemahan Bahasa Inggris
```

### Template File Bahasa Bawaan (`lang/id.json`):
```json
{
  "welcome": "Selamat datang di aplikasi kami!",
  "about": "Tentang Kami",
  "welcome_user": "Selamat datang kembali, {name}!",
  "auth": {
    "failed": "Kredensial yang dimasukkan tidak cocok dengan catatan kami.",
    "throttle": "Terlalu banyak percobaan masuk. Silakan coba lagi dalam {seconds} detik."
  }
}
```

---

## 🚀 Panduan Integrasi Cepat

### 1. Inisialisasi Saat Server Startup (`main.rs`)

Panggil fungsi `init` dari `rustbasic_translatable` di dalam fungsi `main` aplikasi Anda untuk memuat semua file JSON ke memori:

```rust
use rustbasic_core::dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 1. Muat seluruh file JSON dari direktori "lang" ke dalam memori
    rustbasic_translatable::init("lang").expect("❌ Gagal menginisialisasi folder bahasa");

    // 2. Tentukan fallback bahasa default secara global
    rustbasic_translatable::set_default_locale("id");

    // Setup server RustBasic seperti biasa...
}
```

### 2. Pasang Middleware Translasi (`routes/web.rs`)

Tambahkan `translatable_middleware` ke router aplikasi Anda. Middleware ini secara otomatis mengekstrak preferensi bahasa pengguna di setiap request dan mengikatnya ke konteks asinkron aktif:

```rust
use rustbasic_core::{Router, get, from_fn};
use crate::app::http::controllers::welcome_controller;

pub fn router() -> Router {
    Router::new()
        .route("/", get(welcome_controller::index))
        .route("/about", get(welcome_controller::about))
        // Daftarkan middleware di sini agar aktif di seluruh rute
        .layer(from_fn(rustbasic_translatable::translatable_middleware))
}
```

---

## 💡 Contoh Penggunaan di Controller

Setelah middleware dipasang, Anda dapat langsung menggunakan makro/fungsi global `__` atau `trans` di mana saja tanpa perlu membawa-bawa argumen locale!

```rust
use rustbasic_core::{Response, ResponseHelper};
use rustbasic_translatable::{__, trans_with};

pub async fn index() -> Response {
    // 1. Memanggil terjemahan datar (flat key)
    let welcome_msg = __("welcome"); 
    // ID => "Selamat datang di aplikasi kami!"
    // EN => "Welcome to our application!"

    // 2. Memanggil terjemahan bersarang (nested key)
    let auth_error = __("auth.failed"); 
    // ID => "Kredensial yang dimasukkan tidak cocok dengan catatan kami."
    // EN => "These credentials do not match our records."

    // 3. Menggunakan placeholder dinamis secara aman
    let greet_user = trans_with("welcome_user", &[("name", "Hendra")]); 
    // ID => "Selamat datang kembali, Hendra!"
    // EN => "Welcome back, Hendra!"

    let throttle_msg = trans_with("auth.throttle", &[("seconds", "60")]);
    // ID => "Terlalu banyak percobaan masuk. Silakan coba lagi dalam 60 detik."

    // Kirimkan respon sebagai JSON
    ResponseHelper::json(serde_json::json!({
        "status": "success",
        "message": greet_user,
        "error_hint": auth_error,
        "delay": throttle_msg
    }))
}
```

---

## 🎯 Cara Pengguna Berpindah Bahasa

Middleware deteksi bahasa secara cerdas mengidentifikasi bahasa terpilih dengan alur berikut:

1. **Query URL**: Cukup tambahkan parameter query pada tautan atau navigasi Anda:
   - `http://localhost:4000/?lang=en` (Bahasa Inggris)
   - `http://localhost:4000/?lang=id` (Bahasa Indonesia)
2. **Session**: Atur preferensi bahasa di session user:
   - `session.set("locale", "en")`
3. **Cookie**: Simpan preferensi bahasa pada cookie browser pengguna:
   - `Cookie: lang=en` atau `Cookie: locale=en`
4. **Header Browser**: Jika tidak ada parameter di atas, sistem akan mendeteksi bahasa bawaan sistem operasi/browser pengguna secara otomatis via header `Accept-Language`.

---

## 🔍 Detail Keamanan & Validasi Crate

Kami berkomitmen menjaga performa dan kestabilan kode di tingkat produksi:
- **Zero Warnings**: Paket dikompilasi 100% bersih tanpa ada warning atau error.
- **Rangkaian Tes Mandiri**: Dilengkapi dengan unit test bawaan untuk menjamin keakuratan flattening JSON dan penggantian placeholder:
  ```bash
  cargo test -p rustbasic-translatable
  ```

---

## 📄 Lisensi

Paket ini dirilis di bawah lisensi **MIT**. Anda bebas menyalin, memodifikasi, dan mendistribusikannya untuk proyek pribadi maupun komersial.
