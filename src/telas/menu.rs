use leptos::prelude::*;

use crate::app::Tela;

#[component]
pub fn Menu(tela: RwSignal<Tela>) -> impl IntoView {
    view! {
        <main class="conteudo">
            <h2>"Opções"</h2>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::ExcluirPeriodo)>
                "Excluir lançamentos por período"
            </button>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Configuracoes)>
                "Configurações (nome, CPF, Pix)"
            </button>
            <button class="botao" on:click=move |_| tela.set(Tela::Pendentes)>
                "Voltar"
            </button>
        </main>
    }
}
