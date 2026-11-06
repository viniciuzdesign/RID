mod cpf;
mod db;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};

pub struct Estado {
    db: Mutex<rusqlite::Connection>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    nome: String,
    cpf: String,
    pix: String,
}

fn erro_banco(e: rusqlite::Error) -> String {
    format!("Erro no banco de dados: {e}")
}

#[tauri::command]
fn obter_config(estado: State<Estado>) -> Result<Config, String> {
    let conn = estado.db.lock().map_err(|_| "Banco indisponível".to_string())?;
    Ok(Config {
        nome: db::ler_config(&conn, "nome").map_err(erro_banco)?,
        cpf: db::ler_config(&conn, "cpf").map_err(erro_banco)?,
        pix: db::ler_config(&conn, "pix").map_err(erro_banco)?,
    })
}

#[tauri::command]
fn salvar_config(estado: State<Estado>, config: Config) -> Result<Config, String> {
    let nome = config.nome.split_whitespace().collect::<Vec<_>>().join(" ");
    let pix = config.pix.trim().to_string();

    if nome.is_empty() {
        return Err("Informe seu nome.".into());
    }
    if !cpf::valido(&config.cpf) {
        return Err("CPF inválido. Confira os 11 dígitos.".into());
    }
    if pix.is_empty() {
        return Err("Informe sua chave Pix.".into());
    }

    let salvo = Config {
        nome,
        cpf: cpf::formatar(&config.cpf),
        pix,
    };

    let mut conn = estado.db.lock().map_err(|_| "Banco indisponível".to_string())?;
    let tx = conn.transaction().map_err(erro_banco)?;
    db::gravar_config(&tx, "nome", &salvo.nome).map_err(erro_banco)?;
    db::gravar_config(&tx, "cpf", &salvo.cpf).map_err(erro_banco)?;
    db::gravar_config(&tx, "pix", &salvo.pix).map_err(erro_banco)?;
    tx.commit().map_err(erro_banco)?;

    Ok(salvo)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let pasta = app.path().app_data_dir()?;
            std::fs::create_dir_all(&pasta)?;
            let conn = db::abrir(&pasta.join("rid.db"))?;
            app.manage(Estado {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![obter_config, salvar_config])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o RID");
}
