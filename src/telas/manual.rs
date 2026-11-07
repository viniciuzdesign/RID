//! Lançamento manual (cupom sem QR Code): chave opcional.

use leptos::prelude::*;

use crate::app::Tela;
use crate::telas::formulario::{FormLancamento, Inicial, Modo};

#[component]
pub fn Manual(tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <FormLancamento
            titulo="Lançamento manual"
            inicial=Inicial::default()
            modo=Modo::Manual
            tela=tela
            aviso=aviso
        />
    }
}
