use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Lancamento};
use crate::app::Tela;
use crate::util::{data_br, moeda};

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
                Some(Ok(itens)) if itens.is_empty() => {
                    view! {
                        <section class="cartao vazio">
                            <p>"Nenhum lançamento pendente."</p>
                            <p class="discreto">"Toque em “Lançar cupom” para ler o QR Code."</p>
                        </section>
                    }
                        .into_any()
                }
                Some(Ok(itens)) => {
                    let total: i64 = itens.iter().map(|l| l.valor_centavos).sum();
                    view! {
                        <section class="cartao total">
                            <span>"Total pendente"</span>
                            <strong>{moeda(total)}</strong>
                        </section>
                        <ul class="lista">
                            {itens
                                .into_iter()
                                .map(|l| {
                                    view! {
                                        <li class="item">
                                            <div class="item-info">
                                                <strong>{l.cidade}</strong>
                                                <span class="discreto">{data_br(&l.data)}</span>
                                            </div>
                                            <span class="valor">{moeda(l.valor_centavos)}</span>
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
            <button class="botao" on:click=move |_| tela.set(Tela::Novo)>
                "Lançar cupom"
            </button>
        </div>
    }
}
