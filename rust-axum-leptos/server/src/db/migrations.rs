use std::{cmp::Ordering, collections::HashMap};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use include_dir::{include_dir, Dir};
use rusqlite::{params, Connection};
use tracing::*;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

struct Migration {
    path: String,
    timestamp: DateTime<Utc>,
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
		CREATE TABLE IF NOT EXISTS migrations (
			id INTEGER PRIMARY KEY,
			path STRING NOT NULL UNIQUE,
			timestamp DATETIME NOT NULL
		)
	"#,
    )
    .map_err(|e| anyhow!("failed to create migrations table: {e:?}"))?;

    {
        let mut s = conn.prepare("SELECT path, timestamp FROM migrations ORDER BY timestamp")?;
        let iter = s.query_map([], |row| {
            Ok(Migration {
                path: row.get(0)?,
                timestamp: DateTime::from_timestamp_millis(row.get::<_, i64>(1)?).unwrap(),
            })
        })?;
        for m in iter {
            let m = m?;
            info!("existing migration: {}, {}", m.path, m.timestamp);
        }
    }

    let mut entries = MIGRATIONS_DIR.find("**/*.sql")?.collect::<Vec<_>>();
    entries.sort_by(|a, b| a.path().cmp(b.path()));
    for entry in entries {
        let path = entry.path().to_str().ok_or(anyhow!(
            "error trying to get migration path as string: {entry:?}"
        ))?;
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM migrations WHERE path = ?",
            [path],
            |row| Ok(row.get(0)?),
        )?;
        if count > 0 {
            debug!("migration already run {:?}", entry.path());
            continue;
        }

        let contents = entry
            .as_file()
            .ok_or(anyhow!("unabled to open entry for reading: {entry:?}"))?
            .contents_utf8()
            .ok_or(anyhow!("failed to read file: {entry:?}"))?;
        conn.execute_batch(contents)
            .map_err(|e| anyhow!("failed to execute migration {entry:?}: {e:?}"))?;
        info!("executed entry {entry:?}\n{contents}");

        conn.execute(
            "INSERT INTO migrations (path, timestamp) VALUES (?, ?)",
            params![path, chrono::Utc::now().timestamp_millis()],
        )?;
    }

    Ok(())
}
