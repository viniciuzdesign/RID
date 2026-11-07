//! Exclusão em lote de todos os lançamentos (pendentes e arquivados) de um período.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::app::Tela;
use crate::util::{hoje_iso, moeda};
use crate::{datas, dialogo};

#[component]
pub fn ExcluirPeriodo(tela: RwSignal<Tela>, aviso: RwSignal<Option<String>>) -> impl IntoView {
    let dialogos = dialogo::usar();
    let hoje = hoje_iso();
    let inicio = RwSignal::new(format!("{}01", &hoje[..8]));
    let fim = RwSignal::new(hoje.clone());
    let erro = RwSignal::new(None::<String>);
    let ocupado = RwSignal::new(false);

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
            let resumo = api::resumo_periodo(i.clone(), f.clone()).await;
            ocupado.try_set(false);
            let resumo = match resumo {
                Ok(r) => r,
                Err(e) => {
                    erro.try_set(Some(e));
                    return;
                }
            };
            let quantidade = resumo.pendentes + resumo.arquivados;
            if quantidade == 0 {
                dialogos.avisar("Nenhum lançamento nesse período.");
                return;
            }
            let mensagem = format!(
                "Excluir {quantidade} lançamento(s) de {} a {} ({} pendente(s), {} arquivado(s)), somando {}? Esta ação não pode ser desfeita.",
                datas::br(&i),
                datas::br(&f),
                resumo.pendentes,
                resumo.arquivados,
                moeda(resumo.total_centavos),
            );
            dialogos.confirmar(mensagem, "Excluir", move || {
                let (i, f) = (i.clone(), f.clone());
                spawn_local(async move {
                    match api::excluir_periodo(i, f).await {
                        Ok(n) => {
                            aviso.set(Some(format!("{n} lançamento(s) excluído(s).")));
                            tela.set(Tela::Pendentes);
                        }
                        Err(e) => dialogos.avisar(e),
                    }
                });
            });
        });
    };

    view! {
        <main class="conteudo">
            <h2>"Excluir por período"</h2>
            <p class="discreto">
                "Exclui todos os lançamentos entre as duas datas, pendentes e arquivados."
            </p>
            <form class="cartao formulario" on:submit=continuar>
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
                <button class="botao perigo" type="submit" disabled=move || ocupado.get()>
                    "Continuar"
                </button>
            </form>
            {move || erro.get().map(|e| view! { <p class="erro">{e}</p> })}
            <button class="botao secundario" on:click=move |_| tela.set(Tela::Menu)>
                "Voltar"
            </button>
        </main>
    }
}
