/// Mantém apenas os dígitos do texto.
pub fn somente_digitos(texto: &str) -> String {
    texto.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Valida um CPF (aceita com ou sem pontuação).
pub fn valido(texto: &str) -> bool {
    let d: Vec<u32> = somente_digitos(texto)
        .chars()
        .filter_map(|c| c.to_digit(10))
        .collect();
    if d.len() != 11 || d.iter().all(|&x| x == d[0]) {
        return false;
    }
    let digito = |n: usize| -> u32 {
        let soma: u32 = (0..n).map(|i| d[i] * (n as u32 + 1 - i as u32)).sum();
        let resto = (soma * 10) % 11;
        if resto == 10 {
            0
        } else {
            resto
        }
    };
    digito(9) == d[9] && digito(10) == d[10]
}

/// Formata como 000.000.000-00 (espera um CPF válido).
pub fn formatar(texto: &str) -> String {
    let s = somente_digitos(texto);
    format!("{}.{}.{}-{}", &s[0..3], &s[3..6], &s[6..9], &s[9..11])
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn aceita_cpfs_validos() {
        assert!(valido("529.982.247-25"));
        assert!(valido("52998224725"));
        assert!(valido("111.444.777-35"));
    }

    #[test]
    fn rejeita_cpfs_invalidos() {
        assert!(!valido("529.982.247-24"));
        assert!(!valido("111.111.111-11"));
        assert!(!valido("1234567890"));
        assert!(!valido(""));
    }

    #[test]
    fn formata() {
        assert_eq!(formatar("52998224725"), "529.982.247-25");
    }
}
