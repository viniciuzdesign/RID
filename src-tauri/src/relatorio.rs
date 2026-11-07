//! Texto do relatório de reembolso no modelo aprovado (para colar no WhatsApp).

use crate::datas;
use crate::valor;

pub struct Linha<'a> {
    pub data: &'a str,
    pub cidade: &'a str,
    pub valor_centavos: i64,
}

/// Monta o relatório. As linhas devem vir em ordem cronológica.
pub fn montar(
    nome: &str,
    cpf: &str,
    pix: &str,
    inicio: &str,
    fim: &str,
    linhas: &[Linha],
) -> String {
    let mut texto = format!(
        "Nome: {nome}\nCPF: {cpf}\nChave Pix: {pix}\nPeríodo: {} a {}\n\n",
        datas::br(inicio),
        datas::br(fim)
    );
    for l in linhas {
        texto.push_str(&format!(
            "{} – {} – {}\n",
            datas::br(l.data),
            l.cidade.to_uppercase(),
            valor::formatar(l.valor_centavos)
        ));
    }
    let total: i64 = linhas.iter().map(|l| l.valor_centavos).sum();
    texto.push_str(&format!("\nTotal: {}", valor::formatar(total)));
    texto
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn segue_o_modelo_aprovado() {
        let linhas = [
            Linha { data: "2026-11-03", cidade: "São Paulo", valor_centavos: 3250 },
            Linha { data: "2026-11-04", cidade: "CAMPINAS", valor_centavos: 2800 },
        ];
        let texto = montar(
            "SEU NOME",
            "000.000.000-00",
            "sua-chave",
            "2026-11-01",
            "2026-11-05",
            &linhas,
        );
        let esperado = "Nome: SEU NOME
CPF: 000.000.000-00
Chave Pix: sua-chave
Período: 01/11/2026 a 05/11/2026

03/11/2026 – SÃO PAULO – R$ 32,50
04/11/2026 – CAMPINAS – R$ 28,00

Total: R$ 60,50";
        assert_eq!(texto, esperado);
    }

    #[test]
    fn total_com_milhar() {
        let linhas = [
            Linha { data: "2026-11-03", cidade: "A", valor_centavos: 99_999 },
            Linha { data: "2026-11-04", cidade: "B", valor_centavos: 1 },
        ];
        assert!(montar("N", "C", "P", "2026-11-01", "2026-11-30", &linhas).ends_with("Total: R$ 1.000,00"));
    }
}
