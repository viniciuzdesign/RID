mod chave;
mod cidade;
mod cpf;
mod db;
mod valor;

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

#[derive(Serialize)]
pub struct Analise {
    chave: String,
    cnpj: String,
    existente: Option<db::Lancamento>,
}

#[tauri::command]
fn analisar_qrcode(estado: State<Estado>, conteudo: String) -> Result<Analise, String> {
    let chave = chave::extrair(&conteudo)
        .ok_or("Este QR Code não contém uma chave de cupom fiscal válida.")?;
    let conn = estado.db.lock().map_err(|_| "Banco indisponível".to_string())?;
    let existente = db::buscar_por_chave(&conn, &chave).map_err(erro_banco)?;
    Ok(Analise {
        cnpj: chave::cnpj(&chave),
        chave,
        existente,
    })
}

#[tauri::command]
async fn buscar_cidade(cnpj: String) -> Result<String, String> {
    cidade::buscar(&cnpj).await
}

#[derive(Deserialize)]
pub struct NovoLancamento {
    chave: String,
    data: String,
    cidade: String,
    valor: String,
}

fn data_valida(data: &str) -> bool {
    let b = data.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn normalizar_cidade(cidade: &str) -> Result<String, String> {
    let cidade = cidade.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase();
    if cidade.is_empty() {
        Err("Informe a cidade.".into())
    } else {
        Ok(cidade)
    }
}

#[tauri::command]
fn criar_lancamento(estado: State<Estado>, novo: NovoLancamento) -> Result<db::Lancamento, String> {
    let chave = novo.chave.trim().to_ascii_uppercase();
    if !chave::valida(&chave) {
        return Err("Chave de acesso inválida.".into());
    }
    if !data_valida(&novo.data) {
        return Err("Data inválida.".into());
    }
    let cidade = normalizar_cidade(&novo.cidade)?;
    let centavos = valor::para_centavos(&novo.valor)?;
    let conn = estado.db.lock().map_err(|_| "Banco indisponível".to_string())?;
    db::inserir(&conn, &chave, &novo.data, &cidade, centavos).map_err(erro_banco)
}

#[tauri::command]
fn listar_pendentes(estado: State<Estado>) -> Result<Vec<db::Lancamento>, String> {
    let conn = estado.db.lock().map_err(|_| "Banco indisponível".to_string())?;
    db::listar(&conn, false).map_err(erro_banco)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let construtor = tauri::Builder::default();
    #[cfg(mobile)]
    let construtor = construtor.plugin(tauri_plugin_barcode_scanner::init());

    construtor
        .setup(|app| {
            let pasta = app.path().app_data_dir()?;
            std::fs::create_dir_all(&pasta)?;
            let conn = db::abrir(&pasta.join("rid.db"))?;
            app.manage(Estado {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            obter_config,
            salvar_config,
            analisar_qrcode,
            buscar_cidade,
            criar_lancamento,
            listar_pendentes
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o RID");
}
