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
- Cidade: automática com internet (CNPJ da chave → BrasilAPI → nome acentuado via IBGE; reserva: ReceitaWS). Sem internet ou falha: digitada pelo usuário.
- Cidade sempre gravada em MAIÚSCULAS.
- Valor: sempre digitado pelo usuário ("32,50", "32.50", "R$ 1.250,00"…), gravado em centavos.
- Data: sempre a data do dia do lançamento (fuso do celular).
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
- Excluir um lançamento.
- Excluir em lote todos os lançamentos de um período (data inicial a final), com confirmação.

### Arquivamento
- Manual, sempre por iniciativa do usuário (o app não pergunta após exportar).
- Em lote por período (só pendentes do período).
- Aba "Arquivados": consultar, editar, excluir e desarquivar (um ou o lote de um período).

### Exportação
- Usuário escolhe o período; texto copiado para a área de transferência.
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

## Arquitetura
- **Tauri 2** (núcleo Rust em `src-tauri/`) + **Leptos 0.8 CSR** (interface Rust → WebAssembly em `src/`, empacotada pelo Trunk).
- Interface chama o núcleo via `window.__TAURI__.core.invoke` (`withGlobalTauri: true`), helper `src/api.rs::chamar`.
- Banco: `rusqlite` (bundled), arquivo `rid.db` em `app_data_dir`. Migrações por `PRAGMA user_version` em `src-tauri/src/db.rs`.
  - Tabela `config(chave, valor)`: chaves `nome`, `cpf`, `pix`.
  - Tabela `lancamentos(id, chave, data 'AAAA-MM-DD', cidade, valor_centavos, arquivado 0/1, criado_em)`.
- Módulos do núcleo: `chave.rs` (extração/DV, com testes), `valor.rs` (centavos, com testes), `cpf.rs` (com testes), `cidade.rs` (HTTP com reqwest + rustls), `lib.rs` (comandos Tauri).
- Comandos existentes: `obter_config`, `salvar_config`, `analisar_qrcode`, `buscar_cidade`, `criar_lancamento`, `listar_pendentes`.
- Telas (`src/telas/`): `configuracoes.rs`, `pendentes.rs`, `novo.rs`. Navegação por enum `Tela` em `src/app.rs`.
- QR Code: plugin `tauri-plugin-barcode-scanner` (só mobile), modo `windowed: true` (câmera atrás da WebView transparente; CSS em `.escaneando`). Permissão em `src-tauri/capabilities/mobile.json`.

## Armadilhas já resolvidas (não repetir)
- Comandos de permissão do plugin são `plugin:barcode-scanner|check_permissions` e `plugin:barcode-scanner|request_permissions` (snake_case). Com camelCase o ACL do Tauri bloqueia.
- O `wasm-opt` do Trunk falha com Rust recente (bulk memory): `index.html` usa `data-wasm-opt-params="--all-features"`.
- Para atualizar com estado assíncrono após trocar de tela, usar `try_set` nos sinais do Leptos.

## Build e publicação
- Não há build local. O workflow `.github/workflows/android.yml` roda a cada push na `main`: instala Rust/Java/NDK, `cargo tauri android init`, `cargo tauri icon`, compila APK `aarch64`, assina com `apksigner` e publica em **Releases** (`build-N/RID.apk`).
- Segredos obrigatórios do repositório: `ANDROID_KEYSTORE_BASE64` (PKCS12, alias `rid`) e `ANDROID_KEYSTORE_PASSWORD`. **Não trocar a chave**: com outra chave, o APK não instala por cima e o usuário perde os dados.
- A cada versão, aumentar `version` em `Cargo.toml`, `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json` (o versionCode do Android deriva dela).
- Testes de unidade do núcleo: `cpf.rs`, `chave.rs` e `valor.rs` não dependem de crates e podem ser testados com `rustc --edition 2021 --test <arquivo>`.

## Estado atual (versão 0.3.1)
- [x] Etapa 1 — Esqueleto Tauri + Leptos e build automático (testado no celular).
- [x] Etapa 2 — Banco SQLite e Configurações (testado no celular).
- [~] Etapa 3 — Lançamento por QR Code, cidade automática, valor, aviso de repetido. A 0.3.0 falhou no celular por nomes errados dos comandos de permissão; corrigido na 0.3.1, **ainda não testado no celular**.
- [ ] Etapa 4 — Pendentes agrupados por semana (seg–dom) com acumulados e total; editar, excluir, excluir em lote por período.
- [ ] Etapa 5 — Arquivar em lote por período; aba Arquivados (editar, excluir, desarquivar).
- [ ] Etapa 6 — Exportação para a área de transferência no modelo acima (usar `tauri-plugin-clipboard-manager`).
- [ ] Etapa 7 — Testes no celular e ajustes finais.
