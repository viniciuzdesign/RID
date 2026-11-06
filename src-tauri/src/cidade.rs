//! Descobre a cidade do emitente a partir do CNPJ contido na chave.
//! Fontes: BrasilAPI (com nome acentuado via IBGE) e, como reserva, ReceitaWS.

use serde_json::Value;
use std::time::Duration;

pub async fn buscar(cnpj: &str) -> Result<String, String> {
    let cliente = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("RID-app")
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(cidade) = brasilapi(&cliente, cnpj).await {
        return Ok(cidade.to_uppercase());
    }
    if let Some(cidade) = receitaws(&cliente, cnpj).await {
        return Ok(cidade.to_uppercase());
    }
    Err("Não foi possível obter a cidade. Digite manualmente.".into())
}

async fn obter_json(cliente: &reqwest::Client, url: &str) -> Option<Value> {
    cliente
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()
}

fn texto(valor: &Value, campo: &str) -> Option<String> {
    let s = valor.get(campo)?.as_str()?.trim();
    (!s.is_empty()).then(|| s.to_string())
}

async fn brasilapi(cliente: &reqwest::Client, cnpj: &str) -> Option<String> {
    let dados = obter_json(cliente, &format!("https://brasilapi.com.br/api/cnpj/v1/{cnpj}")).await?;
    let municipio = texto(&dados, "municipio")?;

    let codigo_ibge = dados.get("codigo_municipio_ibge").and_then(|c| {
        c.as_u64()
            .or_else(|| c.as_str().and_then(|s| s.trim().parse().ok()))
    });
    if let Some(codigo) = codigo_ibge {
        if let Some(nome) = nome_ibge(cliente, codigo).await {
            return Some(nome);
        }
    }
    Some(municipio)
}

async fn nome_ibge(cliente: &reqwest::Client, codigo: u64) -> Option<String> {
    let dados = obter_json(
        cliente,
        &format!("https://servicodados.ibge.gov.br/api/v1/localidades/municipios/{codigo}"),
    )
    .await?;
    texto(&dados, "nome")
}

async fn receitaws(cliente: &reqwest::Client, cnpj: &str) -> Option<String> {
    let dados = obter_json(cliente, &format!("https://receitaws.com.br/v1/cnpj/{cnpj}")).await?;
    if dados.get("status").and_then(Value::as_str) == Some("ERROR") {
        return None;
    }
    texto(&dados, "municipio")
}
