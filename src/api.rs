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
