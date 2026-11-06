use leptos::prelude::*;

#[component]
pub fn Pendentes() -> impl IntoView {
    view! {
        <main class="conteudo">
            <section class="cartao vazio">
                <p>"Nenhum lançamento pendente."</p>
                <p class="discreto">"O lançamento de cupons chega na próxima versão."</p>
            </section>
        </main>
    }
}
