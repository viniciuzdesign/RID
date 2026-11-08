//! Tela principal: lançamentos pendentes agrupados por semana (segunda a domingo).

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Lancamento};
use crate::app::Tela;
use crate::datas;
use crate::util::moeda;

struct Semana {
    inicio: String,
    fim: String,
    total: i64,
    itens: Vec<Lancamento>,
}

/// Agrupa por semana; a lista já vem ordenada da data mais recente para a mais antiga.
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

#[component]
pub fn Pendentes(tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    let lista = RwSignal::new(None::<Result<Vec<Lancamento>, String>>);
    spawn_local(async move {
        let resultado = api::listar_pendentes().await;
        lista.try_set(Some(resultado));
    });

    let mensagem = aviso.get_untracked();
    aviso.set(None);

    view! {
        <main class="conteudo com-rodape">
            {mensagem.map(|m| view! { <p class="ok">{m}</p> })}
            {move || match lista.get() {
                None => view! { <p>"Carregando…"</p> }.into_any(),
                Some(Err(e)) => view! { <p class="erro">{e}</p> }.into_any(),
                Some(Ok(itens)) if itens.is_empty() => view! {
                    <section class="cartao vazio">
                        <p>"Nenhum lançamento pendente."</p>
                        <p class="discreto">
                            "Use “Ler QR Code” ou “Manual” para lançar um cupom."
                        </p>
                    </section>
                }
                .into_any(),
                Some(Ok(itens)) => {
                    let total: i64 = itens.iter().map(|l| l.valor_centavos).sum();
                    let semanas = agrupar(itens);
                    view! {
                        <section class="cartao total">
                            <span>"Total pendente"</span>
                            <strong>{moeda(total)}</strong>
                        </section>
                        {semanas
                            .into_iter()
                            .map(|s| {
                                view! {
                                    <section class="semana">
                                        <div class="cabecalho-semana">
                                            <span>
                                                {format!(
                                                    "Semana {} a {}",
                                                    datas::br_curta(&s.inicio),
                                                    datas::br(&s.fim),
                                                )}
                                            </span>
                                            <strong>{moeda(s.total)}</strong>
                                        </div>
                                        <ul class="lista">
                                            {s
                                                .itens
                                                .into_iter()
                                                .map(|l| {
                                                    let id = l.id;
                                                    let sem_chave = l.chave.is_empty();
                                                    view! {
                                                        <li>
                                                            <button
                                                                class="item"
                                                                on:click=move |_| tela.set(Tela::Editar(id))
                                                            >
                                                                <div class="item-info">
                                                                    <strong>{l.cidade}</strong>
                                                                    <span class="discreto">
                                                                        {datas::br(&l.data)}
                                                                        {sem_chave.then(|| " · sem chave")}
                                                                    </span>
                                                                </div>
                                                                <span class="valor">{moeda(l.valor_centavos)}</span>
                                                            </button>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </section>
                                }
                            })
                            .collect_view()}
                    }
                    .into_any()
                }
            }}
        </main>
        <div class="rodape">
            <button class="botao" on:click=move |_| tela.set(Tela::Novo)>
                "Ler QR Code"
            </button>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Manual)>
                "Manual"
            </button>
        </div>
    }
}
