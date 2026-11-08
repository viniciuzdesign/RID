# RID — App de cupons fiscais de almoço (Android, Rust)

Guia para qualquer agente de código (Jules/Gemini, Claude etc.) que continuar este projeto.
Responda e escreva textos do app em **português do Brasil**.

## Regras do dono do projeto
1. Nunca sugerir comandos destrutivos (que parem, removam ou recriem algo funcionando) sem avisar em **negrito** e pedir confirmação.
2. Decidir a abordagem antes de começar e apresentar um plano linear; não mudar de direção no meio. Na dúvida, perguntar antes.
3. Blocos de código contêm somente código; explicações ficam fora deles.
4. Linguagem principal: Rust.

## Objetivo
Guardar no celular a chave de acesso (44 caracteres) dos cupons fiscais (NFC-e) de almoço, como backup do cupom físico, e gerar o relatório de reembolso para colar no WhatsApp do setor financeiro.

## Especificação funcional (aprovada)

### Configurações (cadastro único)
- Nome, CPF (validado) e chave Pix. Usados em todos os relatórios.

### Lançamento
- Leitura do QR Code pela câmera; extração da chave de acesso com validação do dígito verificador (CNPJ pode ser alfanumérico).
- Cidade: automática com internet (CNPJ da chave → BrasilAPI; reserva: ReceitaWS). O nome acentuado vem da lista de municípios da UF no IBGE, numa tentativa de 4 s; se falhar, fica o nome da Receita sem acento. Sem internet: digitada pelo usuário.
- Cidade sempre gravada em MAIÚSCULAS.
- Valor: sempre digitado pelo usuário ("32,50", "32.50", "R$ 1.250,00"…), gravado em centavos.
- Data: sugerida como hoje (fuso do celular) e editável; datas futuras são bloqueadas com mensagem.
- Lançamento manual (cupom sem QR Code): chave NF-e opcional; se digitada, é validada e a cidade é buscada pelo CNPJ.
- Cupom repetido (mesma chave, inclusive arquivado): avisar e só gravar se o usuário confirmar.

### Armazenamento
- Somente no dispositivo (SQLite), persistente. Sem nuvem, sem senha, sem backup em arquivo.
- A chave fica só no app; não vai no relatório.

### Tela principal (pendentes)
- Só lançamentos não arquivados.
- Agrupados por semana de segunda a domingo (pela data do lançamento), com valor acumulado de cada semana.
- Total geral pendente no topo.
- Acumulados são só controle; não entram no relatório.

### Gerenciamento
- Editar qualquer campo (chave, data, cidade, valor) de qualquer lançamento, pendente ou arquivado.
- Na edição (e no manual), botão "Ler chave pelo QR Code": insere só a chave, não muda a cidade, e mostra com botão OK:
  - "Chave NF-e inserida. Emitida na cidade [nome da cidade]."
  - sem conexão/falha: "Chave NF-e inserida. Não foi possível consultar a cidade de emissão."
  - se a chave já existir em outro lançamento, pede confirmação antes.
- Excluir um lançamento.
- Excluir em lote todos os lançamentos (pendentes e arquivados) de um período, com confirmação mostrando quantidade e soma. Início sugerido: dia 1 do mês.

### Arquivamento
- Manual, sempre por iniciativa do usuário (o app não pergunta após exportar).
- Em lote por período (só pendentes do período), com confirmação de quantidade e soma. Início sugerido: pendente mais antigo.
- Aba "Arquivados": consultar, editar, excluir e desarquivar (um ou o lote de um período).

### Exportação
- Usuário escolhe o período (início sugerido: pendente mais antigo; fim: hoje); texto copiado para a área de transferência e mostrado como prévia, com aviso de quantidade e total.
- Somente lançamentos NÃO arquivados (arquivados precisam ser desarquivados antes).
- Cidade em MAIÚSCULAS. Sem título no topo. Modelo exato:

```text
Nome: SEU NOME
CPF: 000.000.000-00
Chave Pix: sua-chave
Período: 01/11/2026 a 05/11/2026

03/11/2026 – SÃO PAULO – R$ 32,50
04/11/2026 – CAMPINAS – R$ 28,00

Total: R$ 60,50
```

## Visual
- Cor predominante laranja. Modo claro e escuro: segue o celular (`prefers-color-scheme`) ou fica fixo em Configurações → Aparência (`Automático`/`Claro`/`Escuro`, salvo na chave `tema` da tabela `config` e aplicado como `data-tema` em `<html>`).
- Tokens de cor no topo de `styles.css` (`--laranja`, `--acao`, `--sobre-acao`, …); o escuro redefine os mesmos tokens. Botões de ação: `--acao` com contraste AA.
- Tela inicial: valor a receber em destaque laranja; lista em linhas de extrato por semana (`src/telas/lista.rs`); rodapé com "Ler QR Code" e "Manual". Avisos e confirmações sobem de baixo da tela; exclusão em vermelho.
- Ícones em SVG de traço em `src/icones.rs`. Fonte do sistema, números tabulares.
- Botão **Exportar** no bloco laranja da tela inicial (leva à tela de exportação, que tem **Exportar tudo** — período do pendente mais antigo ao mais recente — e a escolha de período).

## Gestos e desfazer (0.7.0)
- Arrastar uma linha para a esquerda revela ações (`src/telas/lista.rs`, eventos de ponteiro com `touch-action: pan-y`; só uma linha aberta por vez; tocar fecha):
  - Pendentes: **Arquivar** (verde-escuro #1e6b43) e **Excluir** (vermelho #c62828).
  - Arquivados: **Desarquivar** (cinza-escuro #4a4f57) e **Excluir**.
- Confirmação só no Excluir. Arquivar/Desarquivar vão direto.
- Depois da ação, barra **Desfazer** por 3 s (`src/desfazer.rs`). A ação é feita na hora; "Desfazer" reverte (exclusão é revertida com `restaurar_lancamento`, que recoloca o registro com o mesmo id).

## Arquitetura
- **Tauri 2** (núcleo Rust em `src-tauri/`) + **Leptos 0.8 CSR** (interface Rust → WebAssembly em `src/`, empacotada pelo Trunk).
- Interface chama o núcleo via `window.__TAURI__.core.invoke` (`withGlobalTauri: true`), helper `src/api.rs::chamar`.
- Banco: `rusqlite` (bundled), arquivo `rid.db` em `app_data_dir`. Migrações por `PRAGMA user_version` em `src-tauri/src/db.rs`.
  - Tabela `config(chave, valor)`: chaves `nome`, `cpf`, `pix`.
  - Tabela `lancamentos(id, chave, data 'AAAA-MM-DD', cidade, valor_centavos, arquivado 0/1, criado_em)`.
- Módulos do núcleo: `chave.rs` (extração/DV, com testes), `valor.rs` (centavos, com testes), `cpf.rs` (com testes), `cidade.rs` (HTTP com reqwest + rustls), `lib.rs` (comandos Tauri).
- Comandos: `obter_config`, `salvar_config`, `salvar_tema`, `analisar_qrcode`, `verificar_chave`, `buscar_cidade`, `criar_lancamento`, `atualizar_lancamento`, `obter_lancamento`, `excluir_lancamento`, `resumo_periodo`, `excluir_periodo`, `arquivar_periodo`, `desarquivar_periodo`, `definir_arquivado`, `listar_arquivados`, `exportar`, `exportar_tudo`, `restaurar_lancamento`, `listar_pendentes`.
- `relatorio.rs` monta o texto exato do modelo (com testes); `exportar` copia via `tauri-plugin-clipboard-manager` no núcleo Rust (`app.clipboard().write_text`).
- `src-tauri/src/datas.rs` e `chave.rs` são compartilhados com a interface via `#[path]` em `src/main.rs`.
- Telas (`src/telas/`): `pendentes.rs` (semanas), `novo.rs` (QR), `manual.rs`, `editar.rs`, `formulario.rs` (formulário compartilhado; botão Desarquivar quando arquivado), `arquivados.rs`, `periodo.rs` (exportar, arquivar, desarquivar e excluir por período — `Acao`), `menu.rs`, `configuracoes.rs`. Navegação por enum `Tela` em `src/app.rs`. Diálogos (OK / confirmar) em `src/dialogo.rs`; moldura da câmera em `src/leitor.rs`.
- Formulários usam `novalidate` (a validação é do app, com mensagens em português).
- QR Code: plugin `tauri-plugin-barcode-scanner` (só mobile), modo `windowed: true` (câmera atrás da WebView transparente; CSS em `.escaneando`). Permissão em `src-tauri/capabilities/mobile.json`.

## Armadilhas já resolvidas (não repetir)
- Comandos de permissão do plugin são `plugin:barcode-scanner|check_permissions` e `plugin:barcode-scanner|request_permissions` (snake_case). Com camelCase o ACL do Tauri bloqueia.
- O `wasm-opt` do Trunk falha com Rust recente (bulk memory): `index.html` usa `data-wasm-opt-params="--all-features"`.
- Para atualizar com estado assíncrono após trocar de tela, usar `try_set` nos sinais do Leptos.
- **R8 desligado no release.** O template do Tauri liga a otimização R8, que removia classes do ML Kit (leitor de QR Code) e causava erro "getClass() on a null object reference" seguido de fechamento do app. O workflow troca `enable = true` por `enable = false` em `gen/android/app/build.gradle.kts` logo após o `android init`. Não religar.

## Build e publicação
- Não há build local. O workflow `.github/workflows/android.yml` roda a cada push na `main`: instala Rust/Java/NDK, `cargo tauri android init`, `cargo tauri icon`, compila APK `aarch64`, assina com `apksigner` e publica em **Releases** (`build-N/RID.apk`).
- Segredos obrigatórios do repositório: `ANDROID_KEYSTORE_BASE64` (PKCS12, alias `rid`) e `ANDROID_KEYSTORE_PASSWORD`. **Não trocar a chave**: com outra chave, o APK não instala por cima e o usuário perde os dados.
- A cada versão, aumentar `version` em `Cargo.toml`, `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json` (o versionCode do Android deriva dela).
- Testes de unidade do núcleo: `cpf.rs`, `chave.rs` e `valor.rs` não dependem de crates e podem ser testados com `rustc --edition 2021 --test <arquivo>`.

## Teste automático no emulador
- Branch `diagnostico` + `.github/workflows/diagnostico.yml`: compila o APK x86_64 **depurável**, sobe um emulador Android 15 e executa `ci/teste.py`, que controla a WebView pelo protocolo de depuração do Chrome (uma única conexão; `suppress_origin=True`). Cobre configurações, gestos de arrastar (`adb input swipe`), desfazer, Exportar tudo, manual, data futura, chave com cidade automática, repetida, edição, câmera na edição, exportação, arquivar/desarquivar (individual e por período), exclusão e exclusão por período. Imprime o texto da tela, prints (base64 gzip entre `PRINT_INICIO`/`PRINT_FIM`) e erros do log.
- O emulador fica nos EUA: o IBGE não responde de lá (o acento da cidade só é testável no Brasil).
- O teste desliga a Play Store no emulador: num emulador recém-ligado ela atualiza a WebView em segundo plano e derruba a página (sintomas: "Command … not allowed by ACL" e depurador sem páginas). Não é defeito do app.
- O botão de menu é localizado pelo `aria-label="Opções"`.
- Para validar uma mudança antes de liberar ao usuário: fazer push dela na branch `diagnostico`.

## Estado atual (versão 0.7.0)
- [x] Etapa 1 — Esqueleto Tauri + Leptos e build automático (testado no celular).
- [x] Etapa 2 — Banco SQLite e Configurações (testado no celular).
- [x] Etapa 3 — Lançamento por QR Code (testado no celular com cupom real na 0.3.2).
- [x] Etapa 4 — Semanas, edição, exclusões, data editável, manual com chave opcional, QR na edição (testado no celular na 0.4.0).
- [~] Etapa 5 — Arquivar por período; aba Arquivados; desarquivar individual e por período. Validado no emulador; **falta teste no celular**.
- [~] Etapa 6 — Exportação para a área de transferência no modelo exato. Validado no emulador (texto); **falta confirmar a colagem no WhatsApp**.
- [~] Visual laranja com modo claro/escuro (0.6.0).
- [~] Gestos de arrastar, desfazer em 3 s, Exportar no topo e Exportar tudo (0.7.0). Validado no emulador com gesto real; **falta teste no celular**.
- [ ] Etapa 7 — Testes no celular e ajustes finais.
