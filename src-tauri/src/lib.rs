#[tauri::command]
fn versao() -> String {
    format!("Núcleo Rust OK – versão {}", env!("CARGO_PKG_VERSION"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![versao])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o RID");
}
