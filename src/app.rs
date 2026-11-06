use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Config};
use crate::telas::{configuracoes::Configuracoes, pendentes::Pendentes};

#[derive(Clone, Copy, PartialEq)]
pub enum Tela {
    Pendentes,
    Configuracoes,
}

#[component]
pub fn App() -> impl IntoView {
    let tela = RwSignal::new(Tela::Pendentes);
    let config = RwSignal::new(Config::default());
    let carregado = RwSignal::new(false);
    let erro = RwSignal::new(None::<String>);

    spawn_local(async move {
        match api::obter_config().await {
            Ok(c) => {
                if c.nome.is_empty() {
                    tela.set(Tela::Configuracoes);
                }
                config.set(c);
            }
            Err(e) => erro.set(Some(e)),
        }
        carregado.set(true);
    });

    view! {
        <header class="topo">
            <span>"RID"</span>
            <button
                class="icone"
                aria-label="Configurações"
                on:click=move |_| tela.set(Tela::Configuracoes)
            >
                "⚙"
            </button>
        </header>
        {move || erro.get().map(|e| view! { <p class="erro conteudo">{e}</p> })}
        <Show
            when=move || carregado.get()
            fallback=|| view! { <p class="conteudo">"Carregando…"</p> }
        >
            {move || match tela.get() {
                Tela::Pendentes => view! { <Pendentes /> }.into_any(),
                Tela::Configuracoes => view! { <Configuracoes config=config tela=tela /> }.into_any(),
            }}
        </Show>
    }
}
