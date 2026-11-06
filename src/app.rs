use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

async fn versao_do_nucleo() -> String {
    match invoke("versao", JsValue::NULL).await {
        Ok(valor) => valor.as_string().unwrap_or_else(|| "resposta inválida".into()),
        Err(erro) => format!("erro: {:?}", erro),
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (status, set_status) = signal(String::new());

    let testar = move |_| {
        spawn_local(async move {
            set_status.set(versao_do_nucleo().await);
        });
    };

    view! {
        <header class="topo">"RID"</header>
        <main class="conteudo">
            <section class="cartao">
                <p>"Etapa 1: o app foi instalado e a interface está funcionando."</p>
                <p>"Toque no botão para testar a comunicação com o núcleo em Rust."</p>
            </section>
            <button class="botao" on:click=testar>"Testar núcleo"</button>
            <p class="ok">{move || status.get()}</p>
        </main>
    }
}
