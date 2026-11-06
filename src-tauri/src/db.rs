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
