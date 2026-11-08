//! Ações em lote por período: exportar relatório, arquivar, desarquivar e excluir.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::app::Tela;
use crate::util::{hoje_iso, moeda};
use crate::{datas, dialogo};

#[derive(Clone, Copy, PartialEq)]
pub enum Acao {
    Exportar,
    Arquivar,
    Desarquivar,
    Excluir,
}

impl Acao {
    fn titulo(self) -> &'static str {
        match self {
            Acao::Exportar => "Exportar relatório",
            Acao::Arquivar => "Arquivar por período",
            Acao::Desarquivar => "Desarquivar por período",
            Acao::Excluir => "Excluir por período",
        }
    }

    fn descricao(self) -> &'static str {
        match self {
            Acao::Exportar => {
                "Copia o relatório dos lançamentos pendentes do período para você colar no WhatsApp do financeiro."
            }
            Acao::Arquivar => {
                "Arquiva os lançamentos pendentes do período. Eles saem da tela inicial e podem ser desarquivados depois."
            }
            Acao::Desarquivar => "Devolve para os pendentes os lançamentos arquivados do período.",
            Acao::Excluir => "Exclui todos os lançamentos entre as duas datas, pendentes e arquivados.",
        }
    }

    fn botao(self) -> &'static str {
        match self {
            Acao::Exportar => "Copiar relatório",
            _ => "Continuar",
        }
    }

    fn voltar(self) -> Tela {
        match self {
            Acao::Exportar | Acao::Arquivar => Tela::Pendentes,
            Acao::Desarquivar => Tela::Arquivados,
            Acao::Excluir => Tela::Menu,
        }
    }
}

#[component]
pub fn Periodo(acao: Acao, tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    let dialogos = dialogo::usar();
    let hoje = hoje_iso();
    let inicio = RwSignal::new(format!("{}01", &hoje[..8]));
    let fim = RwSignal::new(hoje.clone());
    let erro = RwSignal::new(None::<String>);
    let ocupado = RwSignal::new(false);
    let previa = RwSignal::new(None::<String>);

    // Para exportar e arquivar, começa pelo lançamento pendente mais antigo.
    if matches!(acao, Acao::Exportar | Acao::Arquivar) {
        spawn_local(async move {
            if let Ok(itens) = api::listar_pendentes().await {
                if let Some(mais_antiga) = itens.iter().map(|l| l.data.clone()).min() {
                    inicio.try_set(mais_antiga);
                }
            }
        });
    }

    let executar = move |i: String, f: String| {
        spawn_local(async move {
            let resultado = match acao {
                Acao::Arquivar => api::arquivar_periodo(i, f)
                    .await
                    .map(|n| (format!("{n} lançamento(s) arquivado(s)."), Tela::Pendentes)),
                Acao::Desarquivar => api::desarquivar_periodo(i, f)
                    .await
                    .map(|n| (format!("{n} lançamento(s) desarquivado(s)."), Tela::Arquivados)),
                Acao::Excluir => api::excluir_periodo(i, f)
                    .await
                    .map(|n| (format!("{n} lançamento(s) excluído(s)."), Tela::Pendentes)),
                Acao::Exportar => return,
            };
            match resultado {
                Ok((mensagem, destino)) => {
                    aviso.set(Some(mensagem));
                    tela.set(destino);
                }
                Err(e) => dialogos.avisar(e),
            }
        });
    };

    let continuar = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (i, f) = (inicio.get_untracked(), fim.get_untracked());
        if !datas::valida(&i) || !datas::valida(&f) {
            erro.set(Some("Informe a data inicial e a data final.".into()));
            return;
        }
        if i > f {
            erro.set(Some("A data inicial precisa ser anterior ou igual à data final.".into()));
            return;
        }
        erro.set(None);
        ocupado.set(true);
        spawn_local(async move {
            if acao == Acao::Exportar {
                let resultado = api::exportar(i, f).await;
                ocupado.try_set(false);
                match resultado {
                    Ok(exp) => {
                        previa.try_set(Some(exp.texto));
                        dialogos.avisar(format!(
                            "Relatório copiado: {} cupom(ns), total {}. Cole no WhatsApp do financeiro.",
                            exp.quantidade,
                            moeda(exp.total_centavos)
                        ));
                    }
                    Err(e) => dialogos.avisar(e),
                }
                return;
            }

            let resumo = api::resumo_periodo(i.clone(), f.clone()).await;
            ocupado.try_set(false);
            let r = match resumo {
                Ok(r) => r,
                Err(e) => {
                    erro.try_set(Some(e));
                    return;
                }
            };
            let periodo = format!("de {} a {}", datas::br(&i), datas::br(&f));
            let (quantidade, mensagem, rotulo) = match acao {
                Acao::Arquivar => (
                    r.pendentes,
                    format!(
                        "Arquivar {} lançamento(s) pendente(s) {periodo}, somando {}? Eles saem da tela inicial e podem ser desarquivados depois.",
                        r.pendentes,
                        moeda(r.total_pendentes)
                    ),
                    "Arquivar",
                ),
                Acao::Desarquivar => (
                    r.arquivados,
                    format!(
                        "Desarquivar {} lançamento(s) {periodo}, somando {}? Eles voltam para a tela inicial.",
                        r.arquivados,
                        moeda(r.total_arquivados)
                    ),
                    "Desarquivar",
                ),
                _ => (
                    r.pendentes + r.arquivados,
                    format!(
                        "Excluir {} lançamento(s) {periodo} ({} pendente(s), {} arquivado(s)), somando {}? Esta ação não pode ser desfeita.",
                        r.pendentes + r.arquivados,
                        r.pendentes,
                        r.arquivados,
                        moeda(r.total_centavos)
                    ),
                    "Excluir",
                ),
            };
            if quantidade == 0 {
                dialogos.avisar(match acao {
                    Acao::Arquivar => "Nenhum lançamento pendente nesse período.",
                    Acao::Desarquivar => "Nenhum lançamento arquivado nesse período.",
                    _ => "Nenhum lançamento nesse período.",
                });
                return;
            }
            dialogos.confirmar(mensagem, rotulo, move || executar(i.clone(), f.clone()));
        });
    };

    let exportar_tudo = move |_| {
        ocupado.set(true);
        spawn_local(async move {
            let resultado = api::exportar_tudo().await;
            ocupado.try_set(false);
            match resultado {
                Ok(exp) => {
                    previa.try_set(Some(exp.texto));
                    dialogos.avisar(format!(
                        "Relatório copiado: {} cupom(ns), total {}. Cole no WhatsApp do financeiro.",
                        exp.quantidade,
                        moeda(exp.total_centavos)
                    ));
                }
                Err(e) => dialogos.avisar(e),
            }
        });
    };

    let classe_botao = match acao {
        Acao::Excluir => "botao perigo",
        Acao::Exportar => "botao secundario",
        _ => "botao",
    };

    view! {
        <main class="conteudo">
            <h2>{acao.titulo()}</h2>
            <p class="discreto">{acao.descricao()}</p>
            {(acao == Acao::Exportar)
                .then(|| {
                    view! {
                        <button class="botao" disabled=move || ocupado.get() on:click=exportar_tudo>
                            "Exportar tudo"
                        </button>
                        <h3 class="ou-periodo">"Ou escolha o período"</h3>
                    }
                })}
            <form class="cartao formulario" novalidate=true on:submit=continuar>
                <label>
                    "Data inicial"
                    <input
                        type="date"
                        max=hoje.clone()
                        prop:value=move || inicio.get()
                        on:input=move |ev| inicio.set(event_target_value(&ev))
                        on:change=move |ev| inicio.set(event_target_value(&ev))
                    />
                </label>
                <label>
                    "Data final"
                    <input
                        type="date"
                        prop:value=move || fim.get()
                        on:input=move |ev| fim.set(event_target_value(&ev))
                        on:change=move |ev| fim.set(event_target_value(&ev))
                    />
                </label>
                <button class=classe_botao type="submit" disabled=move || ocupado.get()>
                    {acao.botao()}
                </button>
            </form>
            {move || erro.get().map(|e| view! { <p class="erro">{e}</p> })}
            {move || {
                previa
                    .get()
                    .map(|texto| {
                        view! {
                            <section class="cartao">
                                <p class="rotulo">"Texto copiado:"</p>
                                <pre class="previa">{texto}</pre>
                            </section>
                        }
                    })
            }}
            <button class="botao secundario" on:click=move |_| tela.set(acao.voltar())>
                "Voltar"
            </button>
        </main>
    }
}
