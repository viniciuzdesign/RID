//! Lista de lançamentos em linhas de extrato, agrupadas por semana (segunda a domingo).
//! Arrastar uma linha para a esquerda revela as ações (arquivar/desarquivar e excluir).

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, Lancamento};
use crate::app::Tela;
use crate::datas;
use crate::desfazer::Desfazer;
use crate::dialogo::Dialogos;
use crate::icones;
use crate::util::moeda;

/// Largura total dos dois botões revelados (2 × 76 px).
const LARGURA_ACOES: f64 = 152.0;
/// Movimento mínimo, em px, para considerar arrasto e não toque.
const LIMIAR: f64 = 8.0;

pub type ListaSinal = RwSignal<Option<Result<Vec<Lancamento>, String>>>;

#[derive(Clone, Copy, PartialEq)]
pub enum Modo {
    Pendentes,
    Arquivados,
}

#[derive(Clone, Copy)]
pub struct Contexto {
    pub tela: RwSignal<Tela>,
    pub modo: Modo,
    pub lista: ListaSinal,
    pub aberto: RwSignal<Option<i64>>,
    pub dialogos: Dialogos,
    pub desfazer: Desfazer,
}

impl Contexto {
    pub fn novo(tela: RwSignal<Tela>, modo: Modo, lista: ListaSinal) -> Self {
        Contexto {
            tela,
            modo,
            lista,
            aberto: RwSignal::new(None),
            dialogos: crate::dialogo::usar(),
            desfazer: crate::desfazer::usar(),
        }
    }

    pub fn recarregar(self) {
        spawn_local(async move {
            let resultado = match self.modo {
                Modo::Pendentes => api::listar_pendentes().await,
                Modo::Arquivados => api::listar_arquivados().await,
            };
            self.lista.try_set(Some(resultado));
        });
    }

    fn tirar_da_lista(self, id: i64) {
        self.aberto.try_set(None);
        self.lista.try_update(|l| {
            if let Some(Ok(itens)) = l {
                itens.retain(|i| i.id != id);
            }
        });
    }

    /// Pendentes → arquivar; arquivados → desarquivar. Sem confirmação, com "Desfazer".
    fn alternar_arquivo(self, id: i64) {
        let arquivar = self.modo == Modo::Pendentes;
        self.tirar_da_lista(id);
        spawn_local(async move {
            if let Err(e) = api::definir_arquivado(id, arquivar).await {
                self.dialogos.avisar(e);
                self.recarregar();
                return;
            }
            self.desfazer.oferecer(
                if arquivar { "Lançamento arquivado." } else { "Lançamento desarquivado." },
                move || {
                    spawn_local(async move {
                        if let Err(e) = api::definir_arquivado(id, !arquivar).await {
                            self.dialogos.avisar(e);
                        }
                        self.recarregar();
                    });
                },
            );
        });
    }

    /// Excluir: pede confirmação, exclui e oferece "Desfazer".
    fn excluir(self, lancamento: Lancamento) {
        self.dialogos.confirmar(
            "Excluir este lançamento? Você terá 3 segundos para desfazer.",
            "Excluir",
            move || {
                let copia = lancamento.clone();
                let id = copia.id;
                self.tirar_da_lista(id);
                spawn_local(async move {
                    if let Err(e) = api::excluir_lancamento(id).await {
                        self.dialogos.avisar(e);
                        self.recarregar();
                        return;
                    }
                    self.desfazer.oferecer("Lançamento excluído.", move || {
                        let copia = copia.clone();
                        spawn_local(async move {
                            if let Err(e) = api::restaurar_lancamento(copia).await {
                                self.dialogos.avisar(e);
                            }
                            self.recarregar();
                        });
                    });
                });
            },
        );
    }
}

struct Semana {
    inicio: String,
    fim: String,
    total: i64,
    itens: Vec<Lancamento>,
}

/// Agrupa por semana; a lista já vem da data mais recente para a mais antiga.
fn agrupar(itens: Vec<Lancamento>) -> Vec<Semana> {
    let mut semanas: Vec<Semana> = Vec::new();
    for item in itens {
        let inicio = datas::inicio_semana(&item.data).unwrap_or_else(|| item.data.clone());
        match semanas.last_mut() {
            Some(s) if s.inicio == inicio => {
                s.total += item.valor_centavos;
                s.itens.push(item);
            }
            _ => {
                let fim = datas::somar_dias(&inicio, 6).unwrap_or_else(|| inicio.clone());
                semanas.push(Semana {
                    inicio,
                    fim,
                    total: item.valor_centavos,
                    itens: vec![item],
                });
            }
        }
    }
    semanas
}

fn linha(l: Lancamento, ctx: Contexto) -> impl IntoView {
    let id = l.id;
    let dia = l.data.get(8..10).unwrap_or("").to_string();
    let semana = datas::dia_semana_curto(&l.data);
    let sem_chave = l.chave.is_empty();

    // Arrasto: posição de início (x do dedo, deslocamento de partida).
    let inicio = StoredValue::new(None::<(f64, f64)>);
    let arrastando = RwSignal::new(false);
    let deslocamento = RwSignal::new(0.0_f64);
    let moveu = StoredValue::new(false);

    let posicao = move || {
        if arrastando.get() {
            deslocamento.get()
        } else if ctx.aberto.get() == Some(id) {
            -LARGURA_ACOES
        } else {
            0.0
        }
    };

    let ao_tocar = move |ev: leptos::ev::PointerEvent| {
        let base = if ctx.aberto.get_untracked() == Some(id) { -LARGURA_ACOES } else { 0.0 };
        inicio.set_value(Some((ev.client_x() as f64, base)));
        moveu.set_value(false);
    };
    let ao_mover = move |ev: leptos::ev::PointerEvent| {
        if let Some((x0, base)) = inicio.get_value() {
            let dx = ev.client_x() as f64 - x0;
            if !arrastando.get_untracked() && dx.abs() < LIMIAR {
                return;
            }
            if !arrastando.get_untracked() {
                arrastando.set(true);
                moveu.set_value(true);
                if ctx.aberto.get_untracked().is_some_and(|a| a != id) {
                    ctx.aberto.set(None);
                }
            }
            deslocamento.set((base + dx).clamp(-LARGURA_ACOES - 24.0, 0.0));
        }
    };
    let ao_soltar = move |_| {
        if arrastando.get_untracked() {
            let abrir = deslocamento.get_untracked() < -LARGURA_ACOES / 2.0;
            ctx.aberto.set(if abrir { Some(id) } else { None });
            arrastando.set(false);
        }
        inicio.set_value(None);
    };
    let ao_cancelar = move |_| {
        arrastando.set(false);
        inicio.set_value(None);
    };
    let ao_clicar = move |_| {
        if moveu.get_value() {
            moveu.set_value(false);
            return;
        }
        if ctx.aberto.get_untracked().is_some() {
            ctx.aberto.set(None);
            return;
        }
        ctx.tela.set(Tela::Editar(id));
    };

    let acao_arquivo = match ctx.modo {
        Modo::Pendentes => view! {
            <button class="acao-linha arquivar" aria-label="Arquivar" on:click=move |_| ctx.alternar_arquivo(id)>
                {icones::arquivar()}
                <span>"Arquivar"</span>
            </button>
        }
        .into_any(),
        Modo::Arquivados => view! {
            <button class="acao-linha desarquivar" aria-label="Desarquivar" on:click=move |_| ctx.alternar_arquivo(id)>
                {icones::desarquivar()}
                <span>"Desarquivar"</span>
            </button>
        }
        .into_any(),
    };
    let para_excluir = l.clone();

    view! {
        <li class="linha-deslizante" class:aberta=move || ctx.aberto.get() == Some(id)>
            <div class="acoes-linha">
                {acao_arquivo}
                <button
                    class="acao-linha excluir"
                    aria-label="Excluir"
                    on:click=move |_| ctx.excluir(para_excluir.clone())
                >
                    {icones::lixeira()}
                    <span>"Excluir"</span>
                </button>
            </div>
            <button
                class="item"
                class:arrastando=move || arrastando.get()
                style:transform=move || format!("translateX({}px)", posicao())
                on:pointerdown=ao_tocar
                on:pointermove=ao_mover
                on:pointerup=ao_soltar
                on:pointercancel=ao_cancelar
                on:click=ao_clicar
            >
                <span class="dia">
                    <strong>{dia}</strong>
                    <small>{semana}</small>
                </span>
                <span class="item-info">
                    <strong>{l.cidade}</strong>
                    {sem_chave.then(|| view! { <span class="etiqueta">"Sem chave"</span> })}
                </span>
                <span class="valor">{moeda(l.valor_centavos)}</span>
            </button>
        </li>
    }
}

pub fn semanas(itens: Vec<Lancamento>, ctx: Contexto) -> impl IntoView {
    agrupar(itens)
        .into_iter()
        .map(|s| {
            let titulo = format!("{} a {}", datas::br_curta(&s.inicio), datas::br(&s.fim));
            view! {
                <section class="semana">
                    <div class="cabecalho-semana">
                        <h3>{titulo}</h3>
                        <span>{moeda(s.total)}</span>
                    </div>
                    <ul class="lista">
                        {s.itens.into_iter().map(|l| linha(l, ctx)).collect_view()}
                    </ul>
                </section>
            }
        })
        .collect_view()
}
