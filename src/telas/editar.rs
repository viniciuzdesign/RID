//! Edição de um lançamento existente.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::app::Tela;
use crate::telas::formulario::{FormLancamento, Inicial, Modo};
use crate::util::valor_para_campo;

#[component]
pub fn Editar(id: i64, tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    let carregado = RwSignal::new(None::<Result<Inicial, String>>);
    spawn_local(async move {
        let resultado = api::obter_lancamento(id).await.map(|l| Inicial {
            id: Some(l.id),
            arquivado: l.arquivado,
            chave: l.chave,
            data: l.data,
            cidade: l.cidade,
            valor: valor_para_campo(l.valor_centavos),
        });
        carregado.try_set(Some(resultado));
    });

    move || match carregado.get() {
        None => view! { <p class="conteudo">"Carregando…"</p> }.into_any(),
        Some(Err(e)) => view! {
            <main class="conteudo">
                <p class="erro">{e}</p>
                <button class="botao secundario" on:click=move |_| tela.set(Tela::Pendentes)>
                    "Voltar"
                </button>
            </main>
        }
        .into_any(),
        Some(Ok(inicial)) => view! {
            <FormLancamento
                titulo="Editar lançamento"
                inicial=inicial
                modo=Modo::Edicao
                tela=tela
                aviso=aviso
            />
        }
        .into_any(),
    }
}
