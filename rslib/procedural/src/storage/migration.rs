// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use chrono::Utc;
use rusqlite::{params, Connection};

use super::schema::MIGRATIONS;
use crate::core::{ProceduralError, Result};

pub struct MigrationRunner;

impl MigrationRunner {
    pub fn run(conn: &mut Connection) -> Result<u32> {
        // Ensure foreign keys are enabled
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            );
            "#,
        )?;

        let current_version: u32 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;

        let mut applied_count = 0;

        for migration in MIGRATIONS {
            if migration.version > current_version {
                let tx = conn.transaction()?;

                tx.execute_batch(migration.sql).map_err(|e| {
                    ProceduralError::Migration(format!(
                        "Failed executing migration {}: {}",
                        migration.version, e
                    ))
                })?;

                tx.execute(
                    "INSERT INTO schema_migrations (version, description, applied_at) VALUES (?1, ?2, ?3)",
                    params![migration.version, migration.description, Utc::now().timestamp()],
                )?;

                tx.commit()?;
                applied_count += 1;
            }
        }

        Ok(applied_count)
    }

    pub fn current_version(conn: &Connection) -> Result<u32> {
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !table_exists {
            return Ok(0);
        }

        let version: u32 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_apply_cleanly() {
        let mut conn = Connection::open_in_memory().unwrap();
        let applied = MigrationRunner::run(&mut conn).unwrap();
        assert!(applied > 0);

        let ver = MigrationRunner::current_version(&conn).unwrap();
        assert_eq!(ver, MIGRATIONS.len() as u32);

        // Second run is idempotent
        let second_applied = MigrationRunner::run(&mut conn).unwrap();
        assert_eq!(second_applied, 0);
    }
}
