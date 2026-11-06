//! Conversão de valores em reais digitados pelo usuário.

const MAXIMO_CENTAVOS: i64 = 10_000_000_00;

/// Converte textos como "32,50", "32.5", "R$ 1.250,00" ou "40" em centavos.
pub fn para_centavos(texto: &str) -> Result<i64, String> {
    let limpo: String = texto
        .trim()
        .trim_start_matches("R$")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if limpo.is_empty() {
        return Err("Informe o valor do cupom.".into());
    }
    if !limpo.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '.') {
        return Err("Valor inválido. Use apenas números, como 32,50.".into());
    }

    let (inteiro, fracao) = if limpo.contains(',') {
        // Vírgula é o separador decimal; pontos são de milhar.
        let mut partes = limpo.splitn(2, ',');
        let inteiro = partes.next().unwrap_or("").replace('.', "");
        let fracao = partes.next().unwrap_or("").to_string();
        (inteiro, fracao)
    } else if let Some(pos) = limpo.rfind('.') {
        // Só pontos: "1.250" e "1.250.000" são milhar; "32.50" é decimal.
        let grupos: Vec<&str> = limpo.split('.').collect();
        let eh_milhar = (1..=3).contains(&grupos[0].len())
            && grupos[1..].iter().all(|g| g.len() == 3);
        if eh_milhar {
            (limpo.replace('.', ""), String::new())
        } else {
            (limpo[..pos].replace('.', ""), limpo[pos + 1..].to_string())
        }
    } else {
        (limpo.clone(), String::new())
    };

    if fracao.contains(',') || fracao.contains('.') || fracao.len() > 2 {
        return Err("Valor inválido. Use no máximo 2 casas decimais.".into());
    }
    let reais: i64 = if inteiro.is_empty() {
        0
    } else {
        inteiro
            .parse()
            .map_err(|_| "Valor inválido.".to_string())?
    };
    let centavos: i64 = match fracao.len() {
        0 => 0,
        1 => fracao.parse::<i64>().unwrap_or(0) * 10,
        _ => fracao.parse::<i64>().unwrap_or(0),
    };
    let total = reais
        .checked_mul(100)
        .and_then(|v| v.checked_add(centavos))
        .ok_or_else(|| "Valor muito alto.".to_string())?;
    if total <= 0 {
        return Err("O valor precisa ser maior que zero.".into());
    }
    if total > MAXIMO_CENTAVOS {
        return Err("Valor muito alto.".into());
    }
    Ok(total)
}

/// Formata centavos como "R$ 1.234,56".
pub fn formatar(centavos: i64) -> String {
    let reais = centavos / 100;
    let resto = centavos % 100;
    let digitos = reais.to_string();
    let mut com_milhar = String::new();
    for (i, c) in digitos.chars().enumerate() {
        if i > 0 && (digitos.len() - i) % 3 == 0 {
            com_milhar.push('.');
        }
        com_milhar.push(c);
    }
    format!("R$ {com_milhar},{resto:02}")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn converte_formatos_comuns() {
        assert_eq!(para_centavos("32,50"), Ok(3250));
        assert_eq!(para_centavos("32.50"), Ok(3250));
        assert_eq!(para_centavos("32.5"), Ok(3250));
        assert_eq!(para_centavos("40"), Ok(4000));
        assert_eq!(para_centavos("R$ 1.250,00"), Ok(125000));
        assert_eq!(para_centavos("1.250"), Ok(125000));
        assert_eq!(para_centavos(",99"), Ok(99));
    }

    #[test]
    fn rejeita_invalidos() {
        assert!(para_centavos("").is_err());
        assert!(para_centavos("abc").is_err());
        assert!(para_centavos("0").is_err());
        assert!(para_centavos("12,345").is_err());
        assert!(para_centavos("-5").is_err());
    }

    #[test]
    fn formata() {
        assert_eq!(formatar(3250), "R$ 32,50");
        assert_eq!(formatar(5), "R$ 0,05");
        assert_eq!(formatar(123456789), "R$ 1.234.567,89");
    }
}
