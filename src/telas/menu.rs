use leptos::prelude::*;

use crate::app::Tela;
use crate::telas::periodo::Acao;

#[component]
pub fn Menu(tela: RwSignal<Tela>) -> impl IntoView {
    view! {
        <main class="conteudo">
            <h2>"Opções"</h2>
            <button class="botao" on:click=move |_| tela.set(Tela::Periodo(Acao::Exportar))>
                "Exportar relatório (WhatsApp)"
            </button>
            <button
                class="botao secundario"
                on:click=move |_| tela.set(Tela::Periodo(Acao::Arquivar))
            >
                "Arquivar por período"
            </button>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Arquivados)>
                "Ver arquivados"
            </button>
            <button
                class="botao secundario"
                on:click=move |_| tela.set(Tela::Periodo(Acao::Excluir))
            >
                "Excluir lançamentos por período"
            </button>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Configuracoes)>
                "Configurações (nome, CPF, Pix)"
            </button>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Pendentes)>
                "Voltar"
            </button>
        </main>
    }
}
