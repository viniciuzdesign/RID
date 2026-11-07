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
pub struct NovoLancamento {
    pub chave: String,
    pub data: String,
    pub cidade: String,
    pub valor: String,
}

#[derive(Serialize)]
struct ArgsConteudo {
    conteudo: String,
}

#[derive(Serialize)]
struct ArgsCnpj {
    cnpj: String,
}

#[derive(Serialize)]
struct ArgsNovo {
    novo: NovoLancamento,
}

pub async fn analisar_qrcode(conteudo: String) -> Result<Analise, String> {
    chamar("analisar_qrcode", &ArgsConteudo { conteudo }).await
}

pub async fn buscar_cidade(cnpj: String) -> Result<String, String> {
    chamar("buscar_cidade", &ArgsCnpj { cnpj }).await
}

pub async fn criar_lancamento(novo: NovoLancamento) -> Result<Lancamento, String> {
    chamar("criar_lancamento", &ArgsNovo { novo }).await
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
