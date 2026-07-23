//! Telemetry spool — local append-only SQLite queue (Consumption §7.1, ADR-0002).
//!
//! v1 is spool-only: events accumulate locally and are never sent anywhere
//! (TraceStore intake is P1+). Emitting must never block or fail a CLI command,
//! so callers use [`Spool::emit_best_effort`], which swallows and logs errors.

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// A typed telemetry event. `payload` is a JSON string of event-specific fields.
#[derive(Debug, Clone)]
pub struct Event {
    pub kind: String,
    pub harness: String,
    pub task: String,
    pub bundle_id: String,
    pub ts_ms: u64,
    pub payload: String,
}

impl Event {
    /// Construct an event stamped with the current wall-clock time.
    pub fn now(kind: &str, harness: &str, task: &str, bundle_id: &str, payload: String) -> Event {
        let ts_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Event {
            kind: kind.to_string(),
            harness: harness.to_string(),
            task: task.to_string(),
            bundle_id: bundle_id.to_string(),
            ts_ms,
            payload,
        }
    }
}

/// Append-only local telemetry spool.
pub struct Spool {
    conn: Connection,
}

impl Spool {
    /// Open (creating parent dirs + schema if needed) the spool at `path`.
    pub fn open(path: &Path) -> Result<Spool> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        // Concurrency hardening for the multi-agent consumer model: WAL lets a
        // writer proceed without blocking readers; busy_timeout waits out a lock
        // instead of returning SQLITE_BUSY (which best-effort would silently drop).
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                kind      TEXT NOT NULL,
                harness   TEXT NOT NULL,
                task      TEXT NOT NULL,
                bundle_id TEXT NOT NULL,
                ts_ms     INTEGER NOT NULL,
                payload   TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Spool { conn })
    }

    /// Append one event. Returns an error only on a genuine storage failure.
    pub fn emit(&self, e: &Event) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (kind, harness, task, bundle_id, ts_ms, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![e.kind, e.harness, e.task, e.bundle_id, e.ts_ms, e.payload],
        )?;
        Ok(())
    }

    /// Emit, swallowing and logging any error so a CLI command never fails or
    /// blocks on telemetry (Consumption §7.1: never blocks). Note: a failed
    /// write drops *this telemetry event* — there is no retry yet (P1: durable
    /// flush). The command's real work and the source corpus are unaffected.
    pub fn emit_best_effort(&self, e: &Event) {
        if let Err(err) = self.emit(e) {
            eprintln!("loam: telemetry event dropped (spool write failed, no retry yet): {err}");
        }
    }

    /// Number of spooled events (test/health support).
    pub fn count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))?)
    }

    /// Count spooled events of a given kind (test/health support).
    pub fn count_kind(&self, kind: &str) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM events WHERE kind = ?1", [kind], |r| {
                r.get(0)
            })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_spool() -> (tempfile::TempDir, Spool) {
        let dir = tempfile::tempdir().unwrap();
        let spool = Spool::open(&dir.path().join(".loam-cache/spool.sqlite")).unwrap();
        (dir, spool)
    }

    #[test]
    fn emit_appends_a_row() {
        let (_d, spool) = temp_spool();
        assert_eq!(spool.count().unwrap(), 0);
        spool
            .emit(&Event::now(
                "concept_read",
                "claude-code",
                "t1",
                "b1",
                "{}".into(),
            ))
            .unwrap();
        assert_eq!(spool.count().unwrap(), 1);
        assert_eq!(spool.count_kind("concept_read").unwrap(), 1);
    }

    #[test]
    fn best_effort_never_panics_and_persists() {
        let (_d, spool) = temp_spool();
        spool.emit_best_effort(&Event::now(
            "search_miss",
            "hermes",
            "t2",
            "b1",
            "{\"q\":\"x\"}".into(),
        ));
        assert_eq!(spool.count_kind("search_miss").unwrap(), 1);
    }

    #[test]
    fn open_uses_wal() {
        let (_d, spool) = temp_spool();
        let mode: String = spool
            .conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }
}
