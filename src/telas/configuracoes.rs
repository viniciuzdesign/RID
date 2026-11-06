use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Config};
use crate::app::Tela;

#[component]
pub fn Configuracoes(config: RwSignal<Config>, tela: RwSignal<Tela>) -> impl IntoView {
    let atual = config.get_untracked();
    let nome = RwSignal::new(atual.nome);
    let cpf = RwSignal::new(atual.cpf);
    let pix = RwSignal::new(atual.pix);
    let mensagem = RwSignal::new(None::<(bool, String)>);
    let salvando = RwSignal::new(false);

    let primeiro_acesso = move || config.with(|c| c.nome.is_empty());

    let salvar = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let novo = Config {
            nome: nome.get_untracked(),
            cpf: cpf.get_untracked(),
            pix: pix.get_untracked(),
        };
        salvando.set(true);
        mensagem.set(None);
        spawn_local(async move {
            match api::salvar_config(novo).await {
                Ok(salvo) => {
                    nome.set(salvo.nome.clone());
                    cpf.set(salvo.cpf.clone());
                    pix.set(salvo.pix.clone());
                    config.set(salvo);
                    mensagem.set(Some((true, "Dados salvos.".into())));
                }
                Err(e) => mensagem.set(Some((false, e))),
            }
            salvando.set(false);
        });
    };

    view! {
        <main class="conteudo">
            <h2>"Configurações"</h2>
            <Show when=primeiro_acesso>
                <p class="aviso">
                    "Primeiro acesso: preencha seus dados. Eles aparecem em todos os relatórios."
                </p>
            </Show>
            <form class="cartao formulario" on:submit=salvar>
                <label>
                    "Nome completo"
                    <input
                        type="text"
                        autocomplete="name"
                        prop:value=move || nome.get()
                        on:input=move |ev| nome.set(event_target_value(&ev))
                    />
                </label>
                <label>
                    "CPF"
                    <input
                        type="text"
                        inputmode="numeric"
                        placeholder="000.000.000-00"
                        prop:value=move || cpf.get()
                        on:input=move |ev| cpf.set(event_target_value(&ev))
                    />
                </label>
                <label>
                    "Chave Pix"
                    <input
                        type="text"
                        autocapitalize="off"
                        prop:value=move || pix.get()
                        on:input=move |ev| pix.set(event_target_value(&ev))
                    />
                </label>
                <button class="botao" type="submit" disabled=move || salvando.get()>
                    "Salvar"
                </button>
            </form>
            {move || {
                mensagem
                    .get()
                    .map(|(ok, texto)| {
                        view! { <p class={if ok { "ok" } else { "erro" }}>{texto}</p> }
                    })
            }}
            <Show when=move || !primeiro_acesso()>
                <button class="botao secundario" on:click=move |_| tela.set(Tela::Pendentes)>
                    "Voltar"
                </button>
            </Show>
        </main>
    }
}
