use leptos::prelude::*;

use crate::app::Tela;
use crate::telas::periodo::Acao;

fn opcao(tela: RwSignal<Tela>, destino: Tela, titulo: &'static str, detalhe: &'static str) -> impl IntoView {
    view! {
        <li>
            <button class="opcao" on:click=move |_| tela.set(destino)>
                <strong>{titulo}</strong>
                <span>{detalhe}</span>
            </button>
        </li>
    }
}

#[component]
pub fn Menu(tela: RwSignal<Tela>) -> impl IntoView {
    view! {
        <main class="conteudo">
            <h2>"Opções"</h2>
            <ul class="lista opcoes">
                {opcao(
                    tela,
                    Tela::Periodo(Acao::Exportar),
                    "Exportar relatório",
                    "Copia o texto para colar no WhatsApp do financeiro",
                )}
                {opcao(
                    tela,
                    Tela::Periodo(Acao::Arquivar),
                    "Arquivar por período",
                    "Tira da tela inicial o que já foi reembolsado",
                )}
                {opcao(tela, Tela::Arquivados, "Ver arquivados", "Consultar, editar ou desarquivar")}
            </ul>
            <ul class="lista opcoes">
                {opcao(
                    tela,
                    Tela::Configuracoes,
                    "Configurações",
                    "Nome, CPF, chave Pix e aparência",
                )}
                {opcao(
                    tela,
                    Tela::Periodo(Acao::Excluir),
                    "Excluir por período",
                    "Apaga lançamentos de vez, pendentes e arquivados",
                )}
            </ul>
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Pendentes)>
                "Voltar"
            </button>
        </main>
    }
}
