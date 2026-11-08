/// Data de hoje no fuso do celular, no formato AAAA-MM-DD.
pub fn hoje_iso() -> String {
    let agora = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        agora.get_full_year(),
        agora.get_month() + 1,
        agora.get_date()
    )
}

/// 3250 → "R$ 32,50"
pub fn moeda(centavos: i64) -> String {
    format!("R$ {}", numero(centavos))
}

/// 3250 → "32,50" (para preencher o campo de valor)
pub fn valor_para_campo(centavos: i64) -> String {
    format!("{},{:02}", centavos / 100, centavos % 100)
}

fn numero(centavos: i64) -> String {
    let reais = (centavos / 100).to_string();
    let mut com_milhar = String::new();
    for (i, c) in reais.chars().enumerate() {
        if i > 0 && (reais.len() - i) % 3 == 0 {
            com_milhar.push('.');
        }
        com_milhar.push(c);
    }
    format!("{com_milhar},{:02}", centavos % 100)
}

/// Mantém só letras e números, em maiúsculas (formato da chave NF-e).
pub fn limpar_chave(texto: &str) -> String {
    texto
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

/// Chave em blocos de 4 caracteres, para facilitar a leitura.
pub fn chave_em_blocos(chave: &str) -> String {
    chave
        .as_bytes()
        .chunks(4)
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn descrever_existente(data: &str, arquivado: bool) -> String {
    format!(
        "Este cupom já foi lançado em {} ({}).",
        crate::datas::br(data),
        if arquivado { "arquivado" } else { "pendente" }
    )
}

/// Aplica o tema na página: "claro", "escuro" ou "auto" (segue o celular).
pub fn aplicar_tema(tema: &str) {
    let tema = match tema {
        "claro" | "escuro" => tema,
        _ => "auto",
    };
    let f = js_sys::Function::new_with_args("t", "document.documentElement.dataset.tema = t;");
    let _ = f.call1(&wasm_bindgen::JsValue::NULL, &wasm_bindgen::JsValue::from_str(tema));
}
