mod api;
mod app;
#[path = "../src-tauri/src/chave.rs"]
mod chave;
#[path = "../src-tauri/src/datas.rs"]
mod datas;
mod dialogo;
mod leitor;
mod telas;
mod util;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
