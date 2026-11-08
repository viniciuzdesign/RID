//! Moldura exibida por cima da câmera durante a leitura do QR Code.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;

pub fn mira(ao_cancelar: impl Fn() + Clone + 'static) -> impl IntoView {
    view! {
        <div class="escaneando">
            <p class="dica">"Aponte para o QR Code ou o código de barras da nota"</p>
            <div class="mira"></div>
            <button
                class="botao cancelar-leitura"
                on:click=move |_| {
                    spawn_local(api::cancelar_leitura());
                    ao_cancelar();
                }
            >
                "Cancelar"
            </button>
        </div>
    }
}
