//! Aba "Arquivados": consultar, editar, excluir e desarquivar.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Lancamento};
use crate::app::Tela;
use crate::datas;
use crate::telas::periodo::Acao;
use crate::util::moeda;

#[component]
pub fn Arquivados(tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    let lista = RwSignal::new(None::<Result<Vec<Lancamento>, String>>);
    spawn_local(async move {
        let resultado = api::listar_arquivados().await;
        lista.try_set(Some(resultado));
    });

    let mensagem = aviso.get_untracked();
    aviso.set(None);

    view! {
        <main class="conteudo com-rodape">
            <h2>"Arquivados"</h2>
            {mensagem.map(|m| view! { <p class="ok">{m}</p> })}
            {move || match lista.get() {
                None => view! { <p>"Carregando…"</p> }.into_any(),
                Some(Err(e)) => view! { <p class="erro">{e}</p> }.into_any(),
                Some(Ok(itens)) if itens.is_empty() => view! {
                    <section class="cartao vazio">
                        <p>"Nenhum lançamento arquivado."</p>
                    </section>
                }
                .into_any(),
                Some(Ok(itens)) => {
                    let total: i64 = itens.iter().map(|l| l.valor_centavos).sum();
                    view! {
                        <section class="cartao total">
                            <span>{format!("{} arquivado(s)", itens.len())}</span>
                            <strong>{moeda(total)}</strong>
                        </section>
                        <ul class="lista">
                            {itens
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
                    }
                    .into_any()
                }
            }}
        </main>
        <div class="rodape">
            <button
                class="botao secundario"
                on:click=move |_| tela.set(Tela::Periodo(Acao::Desarquivar))
            >
                "Desarquivar período"
            </button>
            <button class="botao" on:click=move |_| tela.set(Tela::Pendentes)>
                "Voltar"
            </button>
        </div>
    }
}
