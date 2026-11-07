mod chave;
mod cidade;
mod cpf;
mod datas;
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

fn banco<'a>(estado: &'a State<Estado>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    estado.db.lock().map_err(|_| "Banco indisponível".to_string())
}

#[tauri::command(rename_all = "snake_case")]
fn analisar_qrcode(
    estado: State<Estado>,
    conteudo: String,
    ignorar_id: Option<i64>,
) -> Result<Analise, String> {
    let chave = chave::extrair(&conteudo)
        .ok_or("Este QR Code não contém uma chave de cupom fiscal válida.")?;
    let conn = banco(&estado)?;
    let existente = db::buscar_por_chave(&conn, &chave, ignorar_id).map_err(erro_banco)?;
    Ok(Analise {
        cnpj: chave::cnpj(&chave),
        chave,
        existente,
    })
}

/// Lançamento existente com a mesma chave (para avisar sobre cupom repetido).
#[tauri::command(rename_all = "snake_case")]
fn verificar_chave(
    estado: State<Estado>,
    chave: String,
    ignorar_id: Option<i64>,
) -> Result<Option<db::Lancamento>, String> {
    let chave = normalizar_chave(&chave)?;
    let conn = banco(&estado)?;
    db::buscar_por_chave(&conn, &chave, ignorar_id).map_err(erro_banco)
}

#[tauri::command]
async fn buscar_cidade(cnpj: String) -> Result<String, String> {
    cidade::buscar(&cnpj).await
}

#[derive(Deserialize)]
pub struct Dados {
    chave: String,
    data: String,
    cidade: String,
    valor: String,
}

/// Chave opcional: vazia é aceita; se preenchida, precisa ser válida.
fn normalizar_chave(chave: &str) -> Result<String, String> {
    let limpa: String = chave
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase();
    if limpa.is_empty() || chave::valida(&limpa) {
        Ok(limpa)
    } else {
        Err("Chave NF-e inválida. Confira os 44 caracteres ou deixe o campo vazio.".into())
    }
}

fn normalizar_cidade(cidade: &str) -> Result<String, String> {
    let cidade = cidade.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase();
    if cidade.is_empty() {
        Err("Informe a cidade.".into())
    } else {
        Ok(cidade)
    }
}

struct Validado {
    chave: String,
    data: String,
    cidade: String,
    centavos: i64,
}

fn validar(dados: &Dados) -> Result<Validado, String> {
    if !datas::valida(&dados.data) {
        return Err("Data inválida.".into());
    }
    Ok(Validado {
        chave: normalizar_chave(&dados.chave)?,
        data: dados.data.clone(),
        cidade: normalizar_cidade(&dados.cidade)?,
        centavos: valor::para_centavos(&dados.valor)?,
    })
}

#[tauri::command]
fn criar_lancamento(estado: State<Estado>, dados: Dados) -> Result<db::Lancamento, String> {
    let v = validar(&dados)?;
    let conn = banco(&estado)?;
    db::inserir(&conn, &v.chave, &v.data, &v.cidade, v.centavos).map_err(erro_banco)
}

#[tauri::command]
fn atualizar_lancamento(estado: State<Estado>, id: i64, dados: Dados) -> Result<(), String> {
    let v = validar(&dados)?;
    let conn = banco(&estado)?;
    match db::atualizar(&conn, id, &v.chave, &v.data, &v.cidade, v.centavos).map_err(erro_banco)? {
        0 => Err("Lançamento não encontrado.".into()),
        _ => Ok(()),
    }
}

#[tauri::command]
fn obter_lancamento(estado: State<Estado>, id: i64) -> Result<db::Lancamento, String> {
    let conn = banco(&estado)?;
    db::obter(&conn, id)
        .map_err(erro_banco)?
        .ok_or_else(|| "Lançamento não encontrado.".to_string())
}

#[tauri::command]
fn excluir_lancamento(estado: State<Estado>, id: i64) -> Result<(), String> {
    let conn = banco(&estado)?;
    db::excluir(&conn, id).map_err(erro_banco)?;
    Ok(())
}

fn validar_periodo(inicio: &str, fim: &str) -> Result<(), String> {
    if !datas::valida(inicio) || !datas::valida(fim) {
        return Err("Informe a data inicial e a data final.".into());
    }
    if inicio > fim {
        return Err("A data inicial precisa ser anterior ou igual à data final.".into());
    }
    Ok(())
}

#[tauri::command]
fn resumo_periodo(estado: State<Estado>, inicio: String, fim: String) -> Result<db::Resumo, String> {
    validar_periodo(&inicio, &fim)?;
    let conn = banco(&estado)?;
    db::resumo_periodo(&conn, &inicio, &fim).map_err(erro_banco)
}

#[tauri::command]
fn excluir_periodo(estado: State<Estado>, inicio: String, fim: String) -> Result<usize, String> {
    validar_periodo(&inicio, &fim)?;
    let conn = banco(&estado)?;
    db::excluir_periodo(&conn, &inicio, &fim).map_err(erro_banco)
}

#[tauri::command]
fn listar_pendentes(estado: State<Estado>) -> Result<Vec<db::Lancamento>, String> {
    let conn = banco(&estado)?;
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
            verificar_chave,
            buscar_cidade,
            criar_lancamento,
            atualizar_lancamento,
            obter_lancamento,
            excluir_lancamento,
            resumo_periodo,
            excluir_periodo,
            listar_pendentes
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o RID");
}
