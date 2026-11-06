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

/// "2026-11-06" → "06/11/2026"
pub fn data_br(iso: &str) -> String {
    if iso.len() == 10 {
        format!("{}/{}/{}", &iso[8..10], &iso[5..7], &iso[0..4])
    } else {
        iso.to_string()
    }
}

/// 3250 → "R$ 32,50"
pub fn moeda(centavos: i64) -> String {
    let reais = (centavos / 100).to_string();
    let mut com_milhar = String::new();
    for (i, c) in reais.chars().enumerate() {
        if i > 0 && (reais.len() - i) % 3 == 0 {
            com_milhar.push('.');
        }
        com_milhar.push(c);
    }
    format!("R$ {com_milhar},{:02}", centavos % 100)
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
