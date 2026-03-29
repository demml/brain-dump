use std::path::PathBuf;

use rusqlite::Connection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Custom(String),
}

pub type DbResult<T> = Result<T, DbError>;

/// Returns the default DB path: ~/.brain-dump/brain-dump.db
pub fn default_db_path() -> DbResult<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".brain-dump").join("brain-dump.db"))
        .ok_or_else(|| DbError::Custom("cannot determine home directory".to_string()))
}

/// Open (or create) the database, run migrations, enable WAL.
pub fn open_db(path: &std::path::Path) -> DbResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000; PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

/// Initialize an already-open in-memory connection (for tests).
pub fn init_in_memory(conn: &Connection) -> DbResult<()> {
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    migrate(conn)?;
    Ok(())
}

fn migrate(conn: &Connection) -> DbResult<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if version < 1 {
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL CHECK(type IN ('project','phase','task')),
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','completed','archived')),
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                target_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                edge_type TEXT NOT NULL CHECK(edge_type IN ('parent','blocks','related')),
                created_at TEXT NOT NULL,
                UNIQUE(source_id, target_id, edge_type)
            );
            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id, edge_type);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id, edge_type);

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#8ddb9f'
            );

            CREATE TABLE IF NOT EXISTS node_tags (
                node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY(node_id, tag_id)
            );

            CREATE TABLE IF NOT EXISTS status_history (
                id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
                old_status TEXT NOT NULL,
                new_status TEXT NOT NULL,
                changed_at TEXT NOT NULL
            );

            PRAGMA user_version = 1;
            COMMIT;"
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_db_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        migrate(&conn).unwrap();

        let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0)).unwrap();
        assert_eq!(version, 1);

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"nodes".to_string()));
        assert!(tables.contains(&"edges".to_string()));
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"node_tags".to_string()));
        assert!(tables.contains(&"status_history".to_string()));
    }

    #[test]
    fn test_migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // Should not panic
    }
}
