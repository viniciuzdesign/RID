//! Datas no formato AAAA-MM-DD, sem dependências (usado pelo núcleo e pela interface).

/// Converte "AAAA-MM-DD" em (ano, mês, dia), validando o calendário.
pub fn ler(iso: &str) -> Option<(i64, u32, u32)> {
    let b = iso.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let ano: i64 = iso[0..4].parse().ok()?;
    let mes: u32 = iso[5..7].parse().ok()?;
    let dia: u32 = iso[8..10].parse().ok()?;
    if !(1..=12).contains(&mes) || dia == 0 || dia > dias_no_mes(ano, mes) {
        return None;
    }
    Some((ano, mes, dia))
}

pub fn valida(iso: &str) -> bool {
    ler(iso).is_some()
}

fn bissexto(ano: i64) -> bool {
    (ano % 4 == 0 && ano % 100 != 0) || ano % 400 == 0
}

fn dias_no_mes(ano: i64, mes: u32) -> u32 {
    match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if bissexto(ano) => 29,
        _ => 28,
    }
}

/// Dias desde 1970-01-01 (algoritmo de Howard Hinnant).
fn para_dias(ano: i64, mes: u32, dia: u32) -> i64 {
    let a = if mes <= 2 { ano - 1 } else { ano };
    let era = if a >= 0 { a } else { a - 399 } / 400;
    let ano_era = a - era * 400;
    let m = mes as i64;
    let dia_ano = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + dia as i64 - 1;
    let dia_era = ano_era * 365 + ano_era / 4 - ano_era / 100 + dia_ano;
    era * 146097 + dia_era - 719468
}

fn de_dias(dias: i64) -> (i64, u32, u32) {
    let z = dias + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let dia_era = z - era * 146097;
    let ano_era = (dia_era - dia_era / 1460 + dia_era / 36524 - dia_era / 146096) / 365;
    let dia_ano = dia_era - (365 * ano_era + ano_era / 4 - ano_era / 100);
    let mp = (5 * dia_ano + 2) / 153;
    let dia = (dia_ano - (153 * mp + 2) / 5 + 1) as u32;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let ano = ano_era + era * 400 + if mes <= 2 { 1 } else { 0 };
    (ano, mes, dia)
}

fn formatar(ano: i64, mes: u32, dia: u32) -> String {
    format!("{ano:04}-{mes:02}-{dia:02}")
}

/// Soma (ou subtrai) dias de uma data.
pub fn somar_dias(iso: &str, n: i64) -> Option<String> {
    let (a, m, d) = ler(iso)?;
    let (a, m, d) = de_dias(para_dias(a, m, d) + n);
    Some(formatar(a, m, d))
}

/// Segunda-feira da semana (segunda a domingo) que contém a data.
pub fn inicio_semana(iso: &str) -> Option<String> {
    let (a, m, d) = ler(iso)?;
    let dias = para_dias(a, m, d);
    // 1970-01-01 foi quinta-feira: (dias + 3) % 7 dá 0 para segunda.
    let indice = (dias + 3).rem_euclid(7);
    let (a, m, d) = de_dias(dias - indice);
    Some(formatar(a, m, d))
}

/// Dia da semana: 0 = segunda … 6 = domingo.
pub fn dia_semana(iso: &str) -> Option<u32> {
    let (a, m, d) = ler(iso)?;
    Some((para_dias(a, m, d) + 3).rem_euclid(7) as u32)
}

/// Abreviação do dia da semana em português.
pub fn dia_semana_curto(iso: &str) -> &'static str {
    match dia_semana(iso) {
        Some(0) => "seg",
        Some(1) => "ter",
        Some(2) => "qua",
        Some(3) => "qui",
        Some(4) => "sex",
        Some(5) => "sáb",
        Some(6) => "dom",
        _ => "",
    }
}

/// "2026-11-07" → "07/11/2026"
pub fn br(iso: &str) -> String {
    match ler(iso) {
        Some((a, m, d)) => format!("{d:02}/{m:02}/{a:04}"),
        None => iso.to_string(),
    }
}

/// "2026-11-07" → "07/11"
pub fn br_curta(iso: &str) -> String {
    match ler(iso) {
        Some((_, m, d)) => format!("{d:02}/{m:02}"),
        None => iso.to_string(),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn valida_calendario() {
        assert!(valida("2026-11-07"));
        assert!(valida("2024-02-29"));
        assert!(!valida("2026-02-29"));
        assert!(!valida("2026-13-01"));
        assert!(!valida("2026-11-31"));
        assert!(!valida("07/11/2026"));
        assert!(!valida(""));
    }

    #[test]
    fn soma_dias() {
        assert_eq!(somar_dias("2026-11-07", 1).unwrap(), "2026-11-08");
        assert_eq!(somar_dias("2026-12-31", 1).unwrap(), "2027-01-01");
        assert_eq!(somar_dias("2024-03-01", -1).unwrap(), "2024-02-29");
        assert_eq!(somar_dias("1970-01-01", 0).unwrap(), "1970-01-01");
    }

    #[test]
    fn semana_comeca_na_segunda() {
        // 07/11/2026 é sábado; a semana começa em 02/11/2026 (segunda).
        assert_eq!(inicio_semana("2026-11-07").unwrap(), "2026-11-02");
        assert_eq!(inicio_semana("2026-11-08").unwrap(), "2026-11-02"); // domingo
        assert_eq!(inicio_semana("2026-11-09").unwrap(), "2026-11-09"); // segunda
        assert_eq!(inicio_semana("2027-01-01").unwrap(), "2026-12-28"); // sexta
    }

    #[test]
    fn dias_da_semana() {
        assert_eq!(dia_semana_curto("2026-11-07"), "sáb");
        assert_eq!(dia_semana_curto("2026-11-09"), "seg");
        assert_eq!(dia_semana_curto("2026-11-08"), "dom");
    }

    #[test]
    fn formata() {
        assert_eq!(br("2026-11-07"), "07/11/2026");
        assert_eq!(br_curta("2026-11-07"), "07/11");
    }
}
