use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

/// Abre (ou cria) o banco local e aplica as migrações.
pub fn abrir(caminho: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(caminho)?;
    migrar(&conn)?;
    Ok(conn)
}

fn migrar(conn: &Connection) -> rusqlite::Result<()> {
    let versao: i32 = conn.query_row("PRAGMA user_version", [], |linha| linha.get(0))?;
    if versao < 1 {
        conn.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS config (
                 chave TEXT PRIMARY KEY,
                 valor TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS lancamentos (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 chave TEXT NOT NULL,
                 data TEXT NOT NULL,
                 cidade TEXT NOT NULL DEFAULT '',
                 valor_centavos INTEGER NOT NULL,
                 arquivado INTEGER NOT NULL DEFAULT 0,
                 criado_em TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_lanc_arquivado_data ON lancamentos (arquivado, data);
             CREATE INDEX IF NOT EXISTS idx_lanc_chave ON lancamentos (chave);
             PRAGMA user_version = 1;
             COMMIT;",
        )?;
    }
    Ok(())
}

pub fn ler_config(conn: &Connection, chave: &str) -> rusqlite::Result<String> {
    let valor: Option<String> = conn
        .query_row(
            "SELECT valor FROM config WHERE chave = ?1",
            params![chave],
            |linha| linha.get(0),
        )
        .optional()?;
    Ok(valor.unwrap_or_default())
}

pub fn gravar_config(conn: &Connection, chave: &str, valor: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO config (chave, valor) VALUES (?1, ?2)
         ON CONFLICT(chave) DO UPDATE SET valor = excluded.valor",
        params![chave, valor],
    )?;
    Ok(())
}

#[derive(serde::Serialize, Clone)]
pub struct Lancamento {
    pub id: i64,
    pub chave: String,
    pub data: String,
    pub cidade: String,
    pub valor_centavos: i64,
    pub arquivado: bool,
}

fn ler_lancamento(linha: &rusqlite::Row) -> rusqlite::Result<Lancamento> {
    Ok(Lancamento {
        id: linha.get(0)?,
        chave: linha.get(1)?,
        data: linha.get(2)?,
        cidade: linha.get(3)?,
        valor_centavos: linha.get(4)?,
        arquivado: linha.get::<_, i64>(5)? != 0,
    })
}

const CAMPOS: &str = "id, chave, data, cidade, valor_centavos, arquivado";

/// Lançamento mais recente com a mesma chave, se houver.
pub fn buscar_por_chave(conn: &Connection, chave: &str) -> rusqlite::Result<Option<Lancamento>> {
    conn.query_row(
        &format!("SELECT {CAMPOS} FROM lancamentos WHERE chave = ?1 ORDER BY id DESC LIMIT 1"),
        params![chave],
        ler_lancamento,
    )
    .optional()
}

pub fn inserir(
    conn: &Connection,
    chave: &str,
    data: &str,
    cidade: &str,
    valor_centavos: i64,
) -> rusqlite::Result<Lancamento> {
    conn.execute(
        "INSERT INTO lancamentos (chave, data, cidade, valor_centavos, arquivado, criado_em)
         VALUES (?1, ?2, ?3, ?4, 0, datetime('now'))",
        params![chave, data, cidade, valor_centavos],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        &format!("SELECT {CAMPOS} FROM lancamentos WHERE id = ?1"),
        params![id],
        ler_lancamento,
    )
}

pub fn listar(conn: &Connection, arquivado: bool) -> rusqlite::Result<Vec<Lancamento>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {CAMPOS} FROM lancamentos WHERE arquivado = ?1 ORDER BY data DESC, id DESC"
    ))?;
    let linhas = stmt.query_map(params![arquivado as i64], ler_lancamento)?;
    linhas.collect()
}
