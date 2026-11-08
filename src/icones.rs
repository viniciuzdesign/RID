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

pub fn arquivar() -> impl IntoView {
    view! {
        <svg class="icone-svg" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M3 4h18v4H3z" />
            <path d="M5 8v12h14V8" />
            <path d="M10 12h4" />
        </svg>
    }
}

pub fn desarquivar() -> impl IntoView {
    view! {
        <svg class="icone-svg" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M3 4h18v4H3z" />
            <path d="M5 8v12h14V8" />
            <path d="M12 18v-7M9 14l3-3 3 3" />
        </svg>
    }
}

pub fn lixeira() -> impl IntoView {
    view! {
        <svg class="icone-svg" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M4 7h16" />
            <path d="M9 7V4h6v3" />
            <path d="M6 7l1 13h10l1-13" />
            <path d="M10 11v6M14 11v6" />
        </svg>
    }
}

pub fn exportar() -> impl IntoView {
    view! {
        <svg class="icone-svg" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 15V4M8 8l4-4 4 4" />
            <path d="M5 12v8h14v-8" />
        </svg>
    }
}
