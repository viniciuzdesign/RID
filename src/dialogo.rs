//! Diálogos de aviso ("OK") e de confirmação, desenhados pela própria página.

use leptos::prelude::*;
use std::sync::Arc;

type Acao = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
struct Dialogo {
    mensagem: String,
    confirmar: Option<(String, Acao)>,
}

#[derive(Clone, Copy)]
pub struct Dialogos(RwSignal<Option<Dialogo>>);

impl Dialogos {
    pub fn novo() -> Self {
        Dialogos(RwSignal::new(None))
    }

    /// Mensagem com um único botão "OK".
    pub fn avisar(&self, mensagem: impl Into<String>) {
        self.0.try_set(Some(Dialogo {
            mensagem: mensagem.into(),
            confirmar: None,
        }));
    }

    /// Mensagem com "Cancelar" e um botão de confirmação que executa `acao`.
    pub fn confirmar(
        &self,
        mensagem: impl Into<String>,
        rotulo: impl Into<String>,
        acao: impl Fn() + Send + Sync + 'static,
    ) {
        self.0.try_set(Some(Dialogo {
            mensagem: mensagem.into(),
            confirmar: Some((rotulo.into(), Arc::new(acao))),
        }));
    }

    fn fechar(&self) {
        self.0.try_set(None);
    }
}

pub fn usar() -> Dialogos {
    expect_context::<Dialogos>()
}

#[component]
pub fn CamadaDialogos() -> impl IntoView {
    let dialogos = usar();
    move || {
        dialogos.0.get().map(|d| {
            let botoes = match d.confirmar {
                Some((rotulo, acao)) => {
                    let classe = if rotulo.starts_with("Excluir") { "botao perigo" } else { "botao" };
                    view! {
                    <button class="botao secundario" on:click=move |_| dialogos.fechar()>
                        "Cancelar"
                    </button>
                    <button
                        class=classe
                        on:click=move |_| {
                            dialogos.fechar();
                            acao();
                        }
                    >
                        {rotulo}
                    </button>
                    }
                    .into_any()
                }
                None => view! {
                    <button class="botao" on:click=move |_| dialogos.fechar()>
                        "OK"
                    </button>
                }
                .into_any(),
            };
            view! {
                <div class="fundo-dialogo">
                    <div class="dialogo" role="dialog">
                        <p>{d.mensagem}</p>
                        <div class="acoes-dialogo">{botoes}</div>
                    </div>
                </div>
            }
        })
    }
}
