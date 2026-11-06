use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Analise, NovoLancamento};
use crate::app::Tela;
use crate::util::{chave_em_blocos, data_br, hoje_iso};

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
    let cidade = RwSignal::new(String::new());
    let buscando_cidade = RwSignal::new(false);
    let aviso_cidade = RwSignal::new(None::<String>);
    let valor = RwSignal::new(String::new());
    let erro_formulario = RwSignal::new(None::<String>);
    let salvando = RwSignal::new(false);
    let data = hoje_iso();

    let abrir_formulario = move |analise: Analise| {
        let cnpj = analise.cnpj.clone();
        etapa.try_set(Etapa::Formulario(analise));
        buscando_cidade.try_set(true);
        aviso_cidade.try_set(None);
        spawn_local(async move {
            let resultado = api::buscar_cidade(cnpj).await;
            match resultado {
                Ok(nome) => {
                    if cidade.try_get_untracked().is_some_and(|c| c.trim().is_empty()) {
                        cidade.try_set(nome);
                    }
                }
                Err(e) => {
                    aviso_cidade.try_set(Some(e));
                }
            }
            buscando_cidade.try_set(false);
        });
    };

    let ler = move || {
        etapa.set(Etapa::Lendo);
        spawn_local(async move {
            let lido = api::ler_qrcode().await;
            match lido {
                Ok(conteudo) => {
                    etapa.try_set(Etapa::Analisando);
                    match api::analisar_qrcode(conteudo).await {
                        Ok(analise) if analise.existente.is_some() => {
                            etapa.try_set(Etapa::Duplicado(analise));
                        }
                        Ok(analise) => abrir_formulario(analise),
                        Err(e) => {
                            etapa.try_set(Etapa::Erro(e));
                        }
                    }
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

    view! {
        {move || match etapa.get() {
            Etapa::Lendo => {
                view! {
                    <div class="escaneando">
                        <p class="dica">"Aponte a câmera para o QR Code do cupom"</p>
                        <div class="mira"></div>
                        <button
                            class="botao cancelar-leitura"
                            on:click=move |_| {
                                spawn_local(api::cancelar_leitura());
                                voltar();
                            }
                        >
                            "Cancelar"
                        </button>
                    </div>
                }
                    .into_any()
            }
            Etapa::Analisando => view! { <p class="conteudo">"Lendo o cupom…"</p> }.into_any(),
            Etapa::Erro(mensagem) => {
                view! {
                    <main class="conteudo">
                        <section class="cartao">
                            <p class="erro">{mensagem}</p>
                        </section>
                        <button class="botao" on:click=move |_| ler()>
                            "Tentar de novo"
                        </button>
                        <button class="botao secundario" on:click=move |_| voltar()>
                            "Voltar"
                        </button>
                    </main>
                }
                    .into_any()
            }
            Etapa::Duplicado(analise) => {
                let anterior = analise.existente.clone();
                let situacao = anterior
                    .as_ref()
                    .map(|l| {
                        format!(
                            "Este cupom já foi lançado em {} ({}).",
                            data_br(&l.data),
                            if l.arquivado { "arquivado" } else { "pendente" },
                        )
                    })
                    .unwrap_or_default();
                view! {
                    <main class="conteudo">
                        <section class="cartao">
                            <p class="aviso">{situacao}</p>
                            <p>"Deseja lançar este cupom novamente?"</p>
                        </section>
                        <button
                            class="botao"
                            on:click=move |_| abrir_formulario(analise.clone())
                        >
                            "Lançar mesmo assim"
                        </button>
                        <button class="botao secundario" on:click=move |_| voltar()>
                            "Cancelar"
                        </button>
                    </main>
                }
                    .into_any()
            }
            Etapa::Formulario(analise) => {
                let chave = analise.chave.clone();
                let data_envio = data.clone();
                let salvar = move |ev: leptos::ev::SubmitEvent| {
                    ev.prevent_default();
                    if salvando.get_untracked() {
                        return;
                    }
                    let novo = NovoLancamento {
                        chave: chave.clone(),
                        data: data_envio.clone(),
                        cidade: cidade.get_untracked(),
                        valor: valor.get_untracked(),
                    };
                    salvando.set(true);
                    erro_formulario.set(None);
                    spawn_local(async move {
                        match api::criar_lancamento(novo).await {
                            Ok(_) => {
                                aviso.set(Some("Cupom lançado.".into()));
                                tela.set(Tela::Pendentes);
                            }
                            Err(e) => {
                                erro_formulario.try_set(Some(e));
                                salvando.try_set(false);
                            }
                        }
                    });
                };
                view! {
                    <main class="conteudo">
                        <h2>"Novo lançamento"</h2>
                        <form class="cartao formulario" on:submit=salvar>
                            <div class="campo">
                                <span class="rotulo">"Data"</span>
                                <span>{data_br(&data)}</span>
                            </div>
                            <div class="campo">
                                <span class="rotulo">"Chave de acesso"</span>
                                <span class="chave">{chave_em_blocos(&analise.chave)}</span>
                            </div>
                            <label>
                                "Cidade"
                                <input
                                    type="text"
                                    autocapitalize="characters"
                                    placeholder=move || {
                                        if buscando_cidade.get() {
                                            "Buscando cidade…"
                                        } else {
                                            "Digite a cidade"
                                        }
                                    }
                                    prop:value=move || cidade.get()
                                    on:input=move |ev| cidade.set(event_target_value(&ev))
                                />
                            </label>
                            {move || {
                                aviso_cidade.get().map(|m| view! { <p class="discreto">{m}</p> })
                            }}
                            <label>
                                "Valor (R$)"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    placeholder="0,00"
                                    prop:value=move || valor.get()
                                    on:input=move |ev| valor.set(event_target_value(&ev))
                                />
                            </label>
                            <button class="botao" type="submit" disabled=move || salvando.get()>
                                "Salvar"
                            </button>
                        </form>
                        {move || erro_formulario.get().map(|e| view! { <p class="erro">{e}</p> })}
                        <button class="botao secundario" on:click=move |_| voltar()>
                            "Cancelar"
                        </button>
                    </main>
                }
                    .into_any()
            }
        }}
    }
}
