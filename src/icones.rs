//! Ícones em SVG (traço), herdam a cor do texto.

use leptos::prelude::*;

pub fn camera() -> impl IntoView {
    view! {
        <svg class="icone-svg" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M4 8h3l2-3h6l2 3h3v11H4z" />
            <circle cx="12" cy="13" r="3.5" />
        </svg>
    }
}

pub fn lapis() -> impl IntoView {
    view! {
        <svg class="icone-svg" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M4 20h4L19 9l-4-4L4 16z" />
            <path d="M13.5 6.5l4 4" />
        </svg>
    }
}

pub fn menu() -> impl IntoView {
    view! {
        <svg class="icone-svg" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M4 7h16M4 12h16M4 17h10" />
        </svg>
    }
}
