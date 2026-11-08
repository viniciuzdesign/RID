//! Barra "Desfazer": aparece por 3 segundos depois de arquivar, desarquivar ou excluir.
//! A ação já foi feita; "Desfazer" executa a reversão.

use leptos::prelude::*;
use std::sync::Arc;
use std::time::Duration;

type Acao = Arc<dyn Fn() + Send + Sync>;

pub const PRAZO: Duration = Duration::from_secs(3);

#[derive(Clone)]
struct Pendente {
    mensagem: String,
    reverter: Acao,
    numero: u64,
}

#[derive(Clone, Copy)]
pub struct Desfazer {
    atual: RwSignal<Option<Pendente>>,
    contador: StoredValue<u64>,
}

impl Desfazer {
    pub fn novo() -> Self {
        Desfazer {
            atual: RwSignal::new(None),
            contador: StoredValue::new(0),
        }
    }

    /// Mostra a barra; se ninguém tocar em "Desfazer" em 3 s, ela some e a ação fica valendo.
    pub fn oferecer(&self, mensagem: impl Into<String>, reverter: impl Fn() + Send + Sync + 'static) {
        let numero = self.contador.get_value() + 1;
        self.contador.set_value(numero);
        self.atual.try_set(Some(Pendente {
            mensagem: mensagem.into(),
            reverter: Arc::new(reverter),
            numero,
        }));
        let atual = self.atual;
        set_timeout(
            move || {
                if atual.try_with_untracked(|p| p.as_ref().map(|p| p.numero)) == Some(Some(numero)) {
                    atual.try_set(None);
                }
            },
            PRAZO,
        );
    }
}

pub fn usar() -> Desfazer {
    expect_context::<Desfazer>()
}

#[component]
pub fn BarraDesfazer() -> impl IntoView {
    let desfazer = usar();
    move || {
        desfazer.atual.get().map(|p| {
            let reverter = p.reverter.clone();
            view! {
                <div class="barra-desfazer" role="status">
                    <span>{p.mensagem}</span>
                    <button
                        class="botao-desfazer"
                        on:click=move |_| {
                            desfazer.atual.set(None);
                            reverter();
                        }
                    >
                        "Desfazer"
                    </button>
                    <span class="prazo"></span>
                </div>
            }
        })
    }
}
