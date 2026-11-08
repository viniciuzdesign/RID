//! Extração e validação da chave de acesso (44 caracteres) de NF-e / NFC-e.
//!
//! Estrutura: cUF(2) AAMM(4) CNPJ(14) modelo(2) série(3) número(9) tpEmis(1) código(8) DV(1).
//! O CNPJ pode ser alfanumérico (padrão da Receita a partir de 2026).

/// Procura no conteúdo do QR Code a primeira sequência de 44 caracteres
/// alfanuméricos que seja uma chave de acesso válida.
pub fn extrair(conteudo: &str) -> Option<String> {
    conteudo
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|parte| parte.len() == 44)
        .map(|parte| parte.to_ascii_uppercase())
        .find(|parte| valida(parte))
}

/// Valida formato e dígito verificador da chave.
pub fn valida(chave: &str) -> bool {
    let b = chave.as_bytes();
    if b.len() != 44 {
        return false;
    }
    let digitos_ok = b[..6].iter().chain(&b[20..]).all(|c| c.is_ascii_digit());
    let cnpj_ok = b[6..20]
        .iter()
        .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase());
    if !digitos_ok || !cnpj_ok {
        return false;
    }
    digito_verificador(&chave[..43]) == u32::from(b[43] - b'0')
}

/// Módulo 11 com pesos de 2 a 9, da direita para a esquerda.
/// Letras valem (código ASCII − 48), como no CNPJ alfanumérico.
pub fn digito_verificador(base: &str) -> u32 {
    let mut peso = 2;
    let mut soma = 0u32;
    for c in base.bytes().rev() {
        soma += u32::from(c - b'0') * peso;
        peso = if peso == 9 { 2 } else { peso + 1 };
    }
    let resto = soma % 11;
    if resto < 2 {
        0
    } else {
        11 - resto
    }
}

/// CNPJ do emitente contido na chave.
pub fn cnpj(chave: &str) -> String {
    chave[6..20].to_string()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn chave_de_teste(base43: &str) -> String {
        format!("{base43}{}", digito_verificador(base43))
    }

    #[test]
    fn extrai_de_url_versao_2() {
        let chave = chave_de_teste("3526110123456700019065001000012345100012345");
        let url = format!(
            "https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx?p={chave}|2|1|1|0A1B2C3D4E5F60718293A4B5C6D7E8F901234567"
        );
        assert_eq!(extrair(&url), Some(chave.clone()));
        assert_eq!(cnpj(&chave), "01234567000190");
    }

    #[test]
    fn extrai_de_url_versao_1() {
        let chave = chave_de_teste("3126110123456700019065001000012345100012345");
        let url = format!("http://nfce.fazenda.mg.gov.br/portalnfce?chNFe={chave}&nVersao=100&tpAmb=1");
        assert_eq!(extrair(&url), Some(chave));
    }

    #[test]
    fn extrai_de_codigo_de_barras() {
        // O código de barras (Code 128) do DANFE contém só os 44 dígitos.
        let chave = chave_de_teste("3526110123456700019055001000012345100012345");
        assert_eq!(extrair(&chave), Some(chave.clone()));
        // Código de barras de produto (EAN-13) não é chave.
        assert_eq!(extrair("7891234567895"), None);
    }

    #[test]
    fn aceita_cnpj_alfanumerico() {
        let chave = chave_de_teste("35261112ABC34501DE3565001000012345100012345");
        assert!(valida(&chave));
        assert_eq!(extrair(&chave.to_lowercase()), Some(chave));
    }

    #[test]
    fn rejeita_dv_errado_e_lixo() {
        let chave = chave_de_teste("3526110123456700019065001000012345100012345");
        let mut errada = chave[..43].to_string();
        let dv = (chave.as_bytes()[43] - b'0' + 1) % 10;
        errada.push(char::from(b'0' + dv));
        assert!(!valida(&errada));
        assert_eq!(extrair("https://exemplo.com/?x=123"), None);
        assert_eq!(extrair(""), None);
    }
}
