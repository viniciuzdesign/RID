//! Formulário de lançamento usado em três situações:
//! - depois de ler o QR Code (chave fixa, cidade buscada automaticamente);
//! - lançamento manual (chave opcional, digitada ou lida pelo QR Code);
//! - edição de um lançamento existente (todos os campos + excluir).

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Analise, Dados};
use crate::app::Tela;
use crate::util::{chave_em_blocos, descrever_existente, hoje_iso, limpar_chave};
use crate::{chave, datas, dialogo, leitor};

#[derive(Clone, Default)]
pub struct Inicial {
    pub id: Option<i64>,
    pub arquivado: bool,
    pub chave: String,
    pub data: String,
    pub cidade: String,
    pub valor: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Modo {
    /// Chave veio do QR Code: aparece fixa e a cidade é buscada ao abrir.
    AposLeitura,
    /// Lançamento manual: chave opcional.
    Manual,
    /// Edição de um lançamento existente.
    Edicao,
}

#[component]
pub fn FormLancamento(
    titulo: &'static str,
    inicial: Inicial,
    modo: Modo,
    tela: RwSignal<Tela>,
    aviso: RwSignal<Option<String>>,
) -> impl IntoView {
    let dialogos = dialogo::usar();
    let id = inicial.id;
    // Lançamentos arquivados voltam para a aba de arquivados.
    let tela_de_volta = if inicial.arquivado { Tela::Arquivados } else { Tela::Pendentes };
    let hoje = hoje_iso();

    let data = RwSignal::new(if inicial.data.is_empty() {
        hoje.clone()
    } else {
        inicial.data.clone()
    });
    let chave_s = RwSignal::new(inicial.chave.clone());
    let cidade = RwSignal::new(inicial.cidade.clone());
    let valor = RwSignal::new(inicial.valor.clone());
    let erro = RwSignal::new(None::<String>);
    let salvando = RwSignal::new(false);
    let buscando_cidade = RwSignal::new(false);
    let aviso_cidade = RwSignal::new(None::<String>);
    let lendo = RwSignal::new(false);

    // Busca a cidade pelo CNPJ da chave e preenche o campo se estiver vazio.
    let preencher_cidade = move |chave: String| {
        buscando_cidade.set(true);
        aviso_cidade.set(None);
        spawn_local(async move {
            match api::buscar_cidade(chave::cnpj(&chave)).await {
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

    if modo == Modo::AposLeitura && chave::valida(&inicial.chave) {
        preencher_cidade(inicial.chave.clone());
    }

    // Digitação manual da chave: ao completar 44 caracteres válidos, busca a cidade.
    let ao_digitar_chave = move |texto: String| {
        let anterior = limpar_chave(&chave_s.get_untracked());
        chave_s.set(texto.clone());
        let limpa = limpar_chave(&texto);
        if limpa != anterior
            && chave::valida(&limpa)
            && cidade.get_untracked().trim().is_empty()
        {
            preencher_cidade(limpa);
        }
    };

    // Chave inserida pela câmera (QR Code ou código de barras): não altera a cidade.
    let aplicar_chave_lida = move |analise: Analise| {
        chave_s.set(analise.chave.clone());
        buscando_cidade.set(true);
        spawn_local(async move {
            let mensagem = match api::buscar_cidade(analise.cnpj).await {
                Ok(nome) => format!("Chave NF-e inserida. Emitida na cidade {nome}."),
                Err(_) => "Chave NF-e inserida. Não foi possível consultar a cidade de emissão.".to_string(),
            };
            buscando_cidade.try_set(false);
            dialogos.avisar(mensagem);
        });
    };

    let ler_qr = move || {
        lendo.set(true);
        spawn_local(async move {
            let lido = api::ler_qrcode().await;
            lendo.try_set(false);
            match lido {
                Ok(conteudo) => match api::analisar_qrcode(conteudo, id).await {
                    Ok(analise) => match analise.existente.clone() {
                        Some(l) => dialogos.confirmar(
                            format!(
                                "{} Inserir a chave mesmo assim?",
                                descrever_existente(&l.data, l.arquivado)
                            ),
                            "Inserir",
                            move || aplicar_chave_lida(analise.clone()),
                        ),
                        None => aplicar_chave_lida(analise),
                    },
                    Err(e) => dialogos.avisar(e),
                },
                Err(e) if e.contains(api::LEITURA_CANCELADA) => {}
                Err(e) => dialogos.avisar(e),
            }
        });
    };

    let gravar = move |dados: Dados| {
        salvando.set(true);
        erro.set(None);
        spawn_local(async move {
            let resultado = match id {
                Some(id) => api::atualizar_lancamento(id, dados).await.map(|_| "Lançamento atualizado."),
                None => api::criar_lancamento(dados).await.map(|_| "Cupom lançado."),
            };
            match resultado {
                Ok(msg) => {
                    aviso.set(Some(msg.into()));
                    tela.set(tela_de_volta);
                }
                Err(e) => {
                    erro.try_set(Some(e));
                    salvando.try_set(false);
                }
            }
        });
    };

    let hoje_guardado = StoredValue::new(hoje.clone());
    let salvar = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if salvando.get_untracked() {
            return;
        }
        let data_escolhida = data.get_untracked();
        if !datas::valida(&data_escolhida) {
            erro.set(Some("Informe a data do cupom.".into()));
            return;
        }
        if data_escolhida > hoje_guardado.get_value() {
            erro.set(Some("A data não pode ser futura.".into()));
            return;
        }
        let limpa = limpar_chave(&chave_s.get_untracked());
        if !limpa.is_empty() && !chave::valida(&limpa) {
            erro.set(Some(
                "Chave NF-e inválida. Confira os 44 caracteres ou deixe o campo vazio.".into(),
            ));
            return;
        }
        let dados = Dados {
            chave: limpa.clone(),
            data: data_escolhida,
            cidade: cidade.get_untracked(),
            valor: valor.get_untracked(),
        };
        // Depois da leitura do QR Code o aviso de repetido já foi dado.
        if modo == Modo::AposLeitura || limpa.is_empty() {
            gravar(dados);
            return;
        }
        salvando.set(true);
        spawn_local(async move {
            match api::verificar_chave(limpa, id).await {
                Ok(Some(l)) => {
                    salvando.try_set(false);
                    dialogos.confirmar(
                        format!(
                            "{} Salvar mesmo assim?",
                            descrever_existente(&l.data, l.arquivado)
                        ),
                        "Salvar",
                        move || gravar(dados.clone()),
                    );
                }
                Ok(None) => gravar(dados),
                Err(e) => {
                    erro.try_set(Some(e));
                    salvando.try_set(false);
                }
            }
        });
    };

    let excluir = move |_| {
        if let Some(id) = id {
            dialogos.confirmar(
                "Excluir este lançamento? Esta ação não pode ser desfeita.",
                "Excluir",
                move || {
                    spawn_local(async move {
                        match api::excluir_lancamento(id).await {
                            Ok(()) => {
                                aviso.set(Some("Lançamento excluído.".into()));
                                tela.set(tela_de_volta);
                            }
                            Err(e) => dialogos.avisar(e),
                        }
                    });
                },
            );
        }
    };

    let desarquivar = move |_| {
        if let Some(id) = id {
            spawn_local(async move {
                match api::definir_arquivado(id, false).await {
                    Ok(()) => {
                        aviso.set(Some("Lançamento desarquivado. Ele voltou para os pendentes.".into()));
                        tela.set(Tela::Arquivados);
                    }
                    Err(e) => dialogos.avisar(e),
                }
            });
        }
    };
    let arquivado = inicial.arquivado;

    let chave_editavel = modo != Modo::AposLeitura;
    let chave_fixa = chave_em_blocos(&inicial.chave);
    let hoje_max = hoje.clone();

    view! {
        <Show
            when=move || !lendo.get()
            fallback=move || leitor::mira(move || lendo.set(false))
        >
            <main class="conteudo">
                <h2>{titulo}</h2>
                <form class="cartao formulario" novalidate=true on:submit=salvar>
                    <label>
                        "Data do cupom"
                        <input
                            type="date"
                            max=hoje_max.clone()
                            prop:value=move || data.get()
                            on:input=move |ev| data.set(event_target_value(&ev))
                            on:change=move |ev| data.set(event_target_value(&ev))
                        />
                    </label>

                    {if chave_editavel {
                        view! {
                            <label>
                                "Chave NF-e (opcional)"
                                <textarea
                                    class="campo-chave"
                                    rows="3"
                                    inputmode="numeric"
                                    autocapitalize="characters"
                                    placeholder="44 números impressos no cupom"
                                    prop:value=move || chave_s.get()
                                    on:input=move |ev| ao_digitar_chave(event_target_value(&ev))
                                ></textarea>
                            </label>
                            <button
                                type="button"
                                class="botao secundario"
                                on:click=move |_| ler_qr()
                            >
                                "Ler chave pela câmera"
                            </button>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="campo">
                                <span class="rotulo">"Chave NF-e"</span>
                                <span class="chave">{chave_fixa.clone()}</span>
                            </div>
                        }
                            .into_any()
                    }}

                    <label>
                        "Cidade"
                        <input
                            type="text"
                            autocapitalize="characters"
                            placeholder=move || {
                                if buscando_cidade.get() { "Buscando cidade…" } else { "Digite a cidade" }
                            }
                            prop:value=move || cidade.get()
                            on:input=move |ev| cidade.set(event_target_value(&ev))
                        />
                    </label>
                    {move || aviso_cidade.get().map(|m| view! { <p class="discreto">{m}</p> })}

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
                {move || erro.get().map(|e| view! { <p class="erro">{e}</p> })}
                {(modo == Modo::Edicao && arquivado)
                    .then(|| {
                        view! {
                            <p class="discreto">"Este lançamento está arquivado."</p>
                            <button class="botao secundario" on:click=desarquivar>
                                "Desarquivar"
                            </button>
                        }
                    })}
                {(modo == Modo::Edicao)
                    .then(|| {
                        view! {
                            <button class="botao perigo" on:click=excluir>
                                "Excluir lançamento"
                            </button>
                        }
                    })}
                <button class="botao secundario" on:click=move |_| tela.set(tela_de_volta)>
                    "Cancelar"
                </button>
            </main>
        </Show>
    }
}
