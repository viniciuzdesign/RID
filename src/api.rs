use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Chama um comando do núcleo Rust e converte a resposta.
pub async fn chamar<A: Serialize, R: DeserializeOwned>(cmd: &str, args: &A) -> Result<R, String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    match invoke(cmd, args).await {
        Ok(valor) => serde_wasm_bindgen::from_value(valor).map_err(|e| e.to_string()),
        Err(erro) => Err(erro.as_string().unwrap_or_else(|| format!("{erro:?}"))),
    }
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct Config {
    pub nome: String,
    pub cpf: String,
    pub pix: String,
}

#[derive(Serialize)]
struct ArgsSalvarConfig {
    config: Config,
}

pub async fn obter_config() -> Result<Config, String> {
    chamar("obter_config", &()).await
}

pub async fn salvar_config(config: Config) -> Result<Config, String> {
    chamar("salvar_config", &ArgsSalvarConfig { config }).await
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Lancamento {
    pub id: i64,
    pub chave: String,
    pub data: String,
    pub cidade: String,
    pub valor_centavos: i64,
    pub arquivado: bool,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Analise {
    pub chave: String,
    pub cnpj: String,
    pub existente: Option<Lancamento>,
}

#[derive(Serialize, Clone)]
pub struct Dados {
    pub chave: String,
    pub data: String,
    pub cidade: String,
    pub valor: String,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Resumo {
    pub pendentes: i64,
    pub arquivados: i64,
    pub total_pendentes: i64,
    pub total_arquivados: i64,
    pub total_centavos: i64,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Exportacao {
    pub texto: String,
    pub quantidade: usize,
    pub total_centavos: i64,
}

#[derive(Serialize)]
struct ArgsArquivado {
    id: i64,
    arquivado: bool,
}

#[derive(Serialize)]
struct ArgsQr {
    conteudo: String,
    ignorar_id: Option<i64>,
}

#[derive(Serialize)]
struct ArgsVerificar {
    chave: String,
    ignorar_id: Option<i64>,
}

#[derive(Serialize)]
struct ArgsCnpj {
    cnpj: String,
}

#[derive(Serialize)]
struct ArgsDados {
    dados: Dados,
}

#[derive(Serialize)]
struct ArgsAtualizar {
    id: i64,
    dados: Dados,
}

#[derive(Serialize)]
struct ArgsId {
    id: i64,
}

#[derive(Serialize)]
struct ArgsPeriodo {
    inicio: String,
    fim: String,
}

pub async fn analisar_qrcode(conteudo: String, ignorar_id: Option<i64>) -> Result<Analise, String> {
    chamar("analisar_qrcode", &ArgsQr { conteudo, ignorar_id }).await
}

pub async fn verificar_chave(chave: String, ignorar_id: Option<i64>) -> Result<Option<Lancamento>, String> {
    chamar("verificar_chave", &ArgsVerificar { chave, ignorar_id }).await
}

pub async fn buscar_cidade(cnpj: String) -> Result<String, String> {
    chamar("buscar_cidade", &ArgsCnpj { cnpj }).await
}

pub async fn criar_lancamento(dados: Dados) -> Result<Lancamento, String> {
    chamar("criar_lancamento", &ArgsDados { dados }).await
}

pub async fn atualizar_lancamento(id: i64, dados: Dados) -> Result<(), String> {
    chamar("atualizar_lancamento", &ArgsAtualizar { id, dados }).await
}

pub async fn obter_lancamento(id: i64) -> Result<Lancamento, String> {
    chamar("obter_lancamento", &ArgsId { id }).await
}

pub async fn excluir_lancamento(id: i64) -> Result<(), String> {
    chamar("excluir_lancamento", &ArgsId { id }).await
}

pub async fn resumo_periodo(inicio: String, fim: String) -> Result<Resumo, String> {
    chamar("resumo_periodo", &ArgsPeriodo { inicio, fim }).await
}

pub async fn excluir_periodo(inicio: String, fim: String) -> Result<usize, String> {
    chamar("excluir_periodo", &ArgsPeriodo { inicio, fim }).await
}

pub async fn arquivar_periodo(inicio: String, fim: String) -> Result<usize, String> {
    chamar("arquivar_periodo", &ArgsPeriodo { inicio, fim }).await
}

pub async fn desarquivar_periodo(inicio: String, fim: String) -> Result<usize, String> {
    chamar("desarquivar_periodo", &ArgsPeriodo { inicio, fim }).await
}

pub async fn definir_arquivado(id: i64, arquivado: bool) -> Result<(), String> {
    chamar("definir_arquivado", &ArgsArquivado { id, arquivado }).await
}

pub async fn listar_arquivados() -> Result<Vec<Lancamento>, String> {
    chamar("listar_arquivados", &()).await
}

pub async fn exportar(inicio: String, fim: String) -> Result<Exportacao, String> {
    chamar("exportar", &ArgsPeriodo { inicio, fim }).await
}

pub async fn listar_pendentes() -> Result<Vec<Lancamento>, String> {
    chamar("listar_pendentes", &()).await
}

// ---------- Leitor de QR Code (plugin barcode-scanner) ----------

#[derive(Deserialize)]
struct Permissoes {
    camera: String,
}

#[derive(Deserialize)]
struct Lido {
    content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpcoesLeitura {
    windowed: bool,
    formats: Vec<&'static str>,
    camera_direction: &'static str,
}

pub const LEITURA_CANCELADA: &str = "cancelled";

/// Pede permissão da câmera (se preciso) e lê um QR Code.
pub async fn ler_qrcode() -> Result<String, String> {
    let atual: Permissoes = chamar("plugin:barcode-scanner|check_permissions", &()).await?;
    if atual.camera != "granted" {
        let pedida: Permissoes =
            chamar("plugin:barcode-scanner|request_permissions", &()).await?;
        if pedida.camera != "granted" {
            return Err(
                "Sem permissão para usar a câmera. Libere a câmera para o RID nas configurações do Android."
                    .into(),
            );
        }
    }
    let lido: Lido = chamar(
        "plugin:barcode-scanner|scan",
        &OpcoesLeitura {
            windowed: true,
            formats: vec!["QR_CODE"],
            camera_direction: "back",
        },
    )
    .await?;
    Ok(lido.content)
}

pub async fn cancelar_leitura() {
    let _ = chamar::<_, ()>("plugin:barcode-scanner|cancel", &()).await;
}
