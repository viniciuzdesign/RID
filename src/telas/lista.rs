//! Lista de lançamentos em linhas de extrato, agrupadas por semana (segunda a domingo).

use leptos::prelude::*;

use crate::api::Lancamento;
use crate::app::Tela;
use crate::datas;
use crate::util::moeda;

struct Semana {
    inicio: String,
    fim: String,
    total: i64,
    itens: Vec<Lancamento>,
}

/// Agrupa por semana; a lista já vem da data mais recente para a mais antiga.
fn agrupar(itens: Vec<Lancamento>) -> Vec<Semana> {
    let mut semanas: Vec<Semana> = Vec::new();
    for item in itens {
        let inicio = datas::inicio_semana(&item.data).unwrap_or_else(|| item.data.clone());
        match semanas.last_mut() {
            Some(s) if s.inicio == inicio => {
                s.total += item.valor_centavos;
                s.itens.push(item);
            }
            _ => {
                let fim = datas::somar_dias(&inicio, 6).unwrap_or_else(|| inicio.clone());
                semanas.push(Semana {
                    inicio,
                    fim,
                    total: item.valor_centavos,
                    itens: vec![item],
                });
            }
        }
    }
    semanas
}

fn linha(l: Lancamento, tela: RwSignal<Tela>) -> impl IntoView {
    let id = l.id;
    let dia = l.data.get(8..10).unwrap_or("").to_string();
    let semana = datas::dia_semana_curto(&l.data);
    let sem_chave = l.chave.is_empty();
    view! {
        <li>
            <button class="item" on:click=move |_| tela.set(Tela::Editar(id))>
                <span class="dia">
                    <strong>{dia}</strong>
                    <small>{semana}</small>
                </span>
                <span class="item-info">
                    <strong>{l.cidade}</strong>
                    {sem_chave.then(|| view! { <span class="etiqueta">"Sem chave"</span> })}
                </span>
                <span class="valor">{moeda(l.valor_centavos)}</span>
            </button>
        </li>
    }
}

pub fn semanas(itens: Vec<Lancamento>, tela: RwSignal<Tela>) -> impl IntoView {
    agrupar(itens)
        .into_iter()
        .map(|s| {
            let titulo = format!("{} a {}", datas::br_curta(&s.inicio), datas::br(&s.fim));
            view! {
                <section class="semana">
                    <div class="cabecalho-semana">
                        <h3>{titulo}</h3>
                        <span>{moeda(s.total)}</span>
                    </div>
                    <ul class="lista">
                        {s.itens.into_iter().map(|l| linha(l, tela)).collect_view()}
                    </ul>
                </section>
            }
        })
        .collect_view()
}
