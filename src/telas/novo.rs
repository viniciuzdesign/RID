//! Lançamento pela leitura do QR Code.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Analise};
use crate::app::Tela;
use crate::leitor;
use crate::telas::formulario::{FormLancamento, Inicial, Modo};
use crate::util::descrever_existente;

#[derive(Clone, PartialEq)]
enum Etapa {
    Lendo,
    Analisando,
    Erro(String),
    Duplicado(Analise),
    Formulario(Analise),
}

#[component]
pub fn Novo(tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    let etapa = RwSignal::new(Etapa::Lendo);

    let ler = move || {
        etapa.set(Etapa::Lendo);
        spawn_local(async move {
            match api::ler_qrcode().await {
                Ok(conteudo) => {
                    etapa.try_set(Etapa::Analisando);
                    let proxima = match api::analisar_qrcode(conteudo, None).await {
                        Ok(a) if a.existente.is_some() => Etapa::Duplicado(a),
                        Ok(a) => Etapa::Formulario(a),
                        Err(e) => Etapa::Erro(e),
                    };
                    etapa.try_set(proxima);
                }
                Err(e) if e.contains(api::LEITURA_CANCELADA) => {}
                Err(e) => {
                    etapa.try_set(Etapa::Erro(e));
                }
            }
        });
    };
    ler();

    let voltar = move || tela.set(Tela::Pendentes);

    move || match etapa.get() {
        Etapa::Lendo => leitor::mira(voltar).into_any(),
        Etapa::Analisando => view! { <p class="conteudo">"Lendo o cupom…"</p> }.into_any(),
        Etapa::Erro(mensagem) => view! {
            <main class="conteudo">
                <section class="cartao">
                    <p class="erro">{mensagem}</p>
                </section>
                <button class="botao" on:click=move |_| ler()>"Tentar de novo"</button>
                <button class="botao secundario" on:click=move |_| voltar()>"Voltar"</button>
            </main>
        }
        .into_any(),
        Etapa::Duplicado(analise) => {
            let situacao = analise
                .existente
                .as_ref()
                .map(|l| descrever_existente(&l.data, l.arquivado))
                .unwrap_or_default();
            view! {
                <main class="conteudo">
                    <section class="cartao">
                        <p class="aviso">{situacao}</p>
                        <p>"Deseja lançar este cupom novamente?"</p>
                    </section>
                    <button
                        class="botao"
                        on:click=move |_| etapa.set(Etapa::Formulario(analise.clone()))
                    >
                        "Lançar mesmo assim"
                    </button>
                    <button class="botao secundario" on:click=move |_| voltar()>"Cancelar"</button>
                </main>
            }
            .into_any()
        }
        Etapa::Formulario(analise) => view! {
            <FormLancamento
                titulo="Novo lançamento"
                inicial=Inicial { chave: analise.chave.clone(), ..Default::default() }
                modo=Modo::AposLeitura
                tela=tela
                aviso=aviso
            />
        }
        .into_any(),
    }
}
