use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub timestamp: i64,
    pub asr_text: String,
    pub final_text: String,
    pub target_app: Option<String>,
}

pub struct HistoryStore {
    conn: Mutex<Connection>,
}

impl HistoryStore {
    pub fn new(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                asr_text TEXT NOT NULL DEFAULT '',
                final_text TEXT NOT NULL DEFAULT '',
                target_app TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp DESC);",
        )?;
        info!("History database opened at {:?}", path);
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn default_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?;
        Ok(config_dir.join("my-mind").join("history.db"))
    }

    pub fn insert(
        &self,
        asr_text: &str,
        final_text: &str,
        target_app: Option<&str>,
    ) -> Result<HistoryRecord> {
        let conn = self.conn.lock().unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO history (id, timestamp, asr_text, final_text, target_app)
             VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4)",
            rusqlite::params![timestamp, asr_text, final_text, target_app],
        )?;

        let id: String =
            conn.query_row("SELECT id FROM history WHERE rowid = last_insert_rowid()", [], |row| {
                row.get(0)
            })?;

        let record = HistoryRecord {
            id,
            timestamp,
            asr_text: asr_text.to_string(),
            final_text: final_text.to_string(),
            target_app: target_app.map(|s| s.to_string()),
        };
        info!("[history] Saved record, id={}", record.id);
        Ok(record)
    }

    pub fn list(&self, limit: u32, offset: u32) -> Result<Vec<HistoryRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, asr_text, final_text, target_app
             FROM history ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2",
        )?;
        let records = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    asr_text: row.get(2)?,
                    final_text: row.get(3)?,
                    target_app: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    pub fn count(&self) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        let count: u64 = conn.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history WHERE id = ?1", rusqlite::params![id])?;
        info!("[history] Deleted record id={}", id);
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history", [])?;
        info!("[history] Cleared all records");
        Ok(())
    }
}
