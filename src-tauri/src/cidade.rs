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
    Some(com_acentos(cliente, &municipio, texto(&dados, "uf").as_deref()).await)
}

async fn receitaws(cliente: &reqwest::Client, cnpj: &str) -> Option<String> {
    let dados = obter_json(cliente, &format!("https://receitaws.com.br/v1/cnpj/{cnpj}")).await?;
    if dados.get("status").and_then(Value::as_str) == Some("ERROR") {
        return None;
    }
    let municipio = texto(&dados, "municipio")?;
    Some(com_acentos(cliente, &municipio, texto(&dados, "uf").as_deref()).await)
}

/// A Receita grava os nomes sem acento ("SAO PAULO"). Procura o nome oficial,
/// acentuado, na lista de municípios da UF no IBGE. Se não achar, devolve o original.
async fn com_acentos(cliente: &reqwest::Client, municipio: &str, uf: Option<&str>) -> String {
    let Some(uf) = uf.filter(|u| u.len() == 2) else {
        return municipio.to_string();
    };
    let uf = uf.to_ascii_uppercase();
    let procurado = sem_acento(municipio);
    let fontes = [
        format!("https://brasilapi.com.br/api/ibge/municipios/v1/{uf}"),
        format!("https://servicodados.ibge.gov.br/api/v1/localidades/estados/{uf}/municipios"),
    ];
    for url in fontes {
        let achado = obter_json(cliente, &url).await.and_then(|lista| {
            lista.as_array()?.iter().find_map(|m| {
                let nome = m.get("nome")?.as_str()?;
                (sem_acento(nome) == procurado).then(|| nome.to_string())
            })
        });
        if let Some(nome) = achado {
            return nome;
        }
    }
    municipio.to_string()
}

/// Maiúsculas, sem acentos e sem espaços repetidos, para comparar nomes.
pub fn sem_acento(texto: &str) -> String {
    texto
        .to_uppercase()
        .chars()
        .map(|c| match c {
            'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'Ç' => 'C',
            'Ñ' => 'N',
            '-' | '\'' | '’' => ' ',
            outro => outro,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod testes {
    use super::sem_acento;

    #[test]
    fn compara_sem_acento() {
        assert_eq!(sem_acento("São Paulo"), "SAO PAULO");
        assert_eq!(sem_acento("Brasília"), "BRASILIA");
        assert_eq!(sem_acento("Embu-Guaçu"), sem_acento("EMBU GUACU"));
        assert_eq!(sem_acento("Pau-d'Arco"), sem_acento("PAU D ARCO"));
    }
}
