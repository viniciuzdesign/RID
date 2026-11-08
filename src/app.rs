use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Config};
use crate::dialogo::{CamadaDialogos, Dialogos};
use crate::telas::{
    arquivados::Arquivados,
    configuracoes::Configuracoes,
    editar::Editar,
    manual::Manual,
    menu::Menu,
    novo::Novo,
    pendentes::Pendentes,
    periodo::{Acao, Periodo},
};

#[derive(Clone, Copy, PartialEq)]
pub enum Tela {
    Pendentes,
    Novo,
    Manual,
    Editar(i64),
    Menu,
    Arquivados,
    Periodo(Acao),
    Configuracoes,
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(Dialogos::novo());

    let tela = RwSignal::new(Tela::Pendentes);
    let config = RwSignal::new(Config::default());
    let carregado = RwSignal::new(false);
    let erro = RwSignal::new(None::<String>);
    let aviso = RwSignal::new(None::<String>);

    spawn_local(async move {
        match api::obter_config().await {
            Ok(c) => {
                crate::util::aplicar_tema(&c.tema);
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
            <button class="icone" aria-label="Opções" on:click=move |_| tela.set(Tela::Menu)>
                {crate::icones::menu()}
            </button>
        </header>
        {move || erro.get().map(|e| view! { <p class="erro conteudo">{e}</p> })}
        <Show
            when=move || carregado.get()
            fallback=|| view! { <p class="conteudo">"Carregando…"</p> }
        >
            {move || match tela.get() {
                Tela::Pendentes => view! { <Pendentes tela=tela aviso=aviso /> }.into_any(),
                Tela::Novo => view! { <Novo tela=tela aviso=aviso /> }.into_any(),
                Tela::Manual => view! { <Manual tela=tela aviso=aviso /> }.into_any(),
                Tela::Editar(id) => view! { <Editar id=id tela=tela aviso=aviso /> }.into_any(),
                Tela::Menu => view! { <Menu tela=tela /> }.into_any(),
                Tela::Arquivados => view! { <Arquivados tela=tela aviso=aviso /> }.into_any(),
                Tela::Periodo(acao) => view! { <Periodo acao=acao tela=tela aviso=aviso /> }.into_any(),
                Tela::Configuracoes => {
                    view! { <Configuracoes config=config tela=tela /> }.into_any()
                }
            }}
        </Show>
        <CamadaDialogos />
    }
}
