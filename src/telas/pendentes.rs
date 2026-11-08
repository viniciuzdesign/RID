//! Tela principal: valor a receber em destaque e lançamentos pendentes por semana.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Lancamento};
use crate::app::Tela;
use crate::icones;
use crate::telas::lista::{self, Contexto, Modo};
use crate::telas::periodo::Acao;
use crate::util::moeda;

#[component]
pub fn Pendentes(tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    let lista_s = RwSignal::new(None::<Result<Vec<Lancamento>, String>>);
    spawn_local(async move {
        let resultado = api::listar_pendentes().await;
        lista_s.try_set(Some(resultado));
    });

    let ctx = Contexto::novo(tela, Modo::Pendentes, lista_s);
    let mensagem = aviso.get_untracked();
    aviso.set(None);

    let resumo = move || {
        lista_s.with(|l| match l {
            Some(Ok(itens)) => (itens.iter().map(|i| i.valor_centavos).sum::<i64>(), itens.len()),
            _ => (0, 0),
        })
    };

    view! {
        <section class="destaque">
            <div class="destaque-topo">
                <span class="destaque-rotulo">"A receber"</span>
                <button
                    class="botao-exportar"
                    on:click=move |_| tela.set(Tela::Periodo(Acao::Exportar))
                >
                    {icones::exportar()}
                    "Exportar"
                </button>
            </div>
            <strong class="destaque-valor">{move || moeda(resumo().0)}</strong>
            <span class="destaque-detalhe">
                {move || match resumo().1 {
                    0 => "Nenhum cupom pendente".to_string(),
                    1 => "1 cupom pendente".to_string(),
                    n => format!("{n} cupons pendentes"),
                }}
            </span>
        </section>
        <main class="conteudo com-rodape">
            {mensagem.map(|m| view! { <p class="ok">{m}</p> })}
            {move || match lista_s.get() {
                None => view! { <p class="discreto">"Carregando…"</p> }.into_any(),
                Some(Err(e)) => view! { <p class="erro">{e}</p> }.into_any(),
                Some(Ok(itens)) if itens.is_empty() => view! {
                    <section class="vazio">
                        <p>"Leia o QR Code ou o código de barras do cupom para guardar a chave e o valor do almoço."</p>
                        <p class="discreto">"Sem código para ler? Use “Manual”."</p>
                    </section>
                }
                .into_any(),
                Some(Ok(itens)) => lista::semanas(itens, ctx).into_any(),
            }}
        </main>
        <div class="rodape">
            <button class="botao principal" on:click=move |_| tela.set(Tela::Novo)>
                {icones::camera()}
                "Ler código"
            </button>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Manual)>
                {icones::lapis()}
                "Manual"
            </button>
        </div>
    }
}
