use std::path::Path;

use chrono::{DateTime, NaiveDate, NaiveDateTime, SubsecRound, Timelike, Utc};
use rusqlite::{Connection, Row, types::Type};
use serde::Serializer;

pub type Ts = DateTime<Utc>;

// Same DDL the Rails schema produced, so an existing Rails DB is used as-is and a fresh DB
// stays readable by the old backend.
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS "lists" ("id" varchar NOT NULL PRIMARY KEY, "uid" varchar NOT NULL, "name" varchar NOT NULL, "position" float NOT NULL, "deleted_at" datetime(6), "revision" integer DEFAULT 0 NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE INDEX IF NOT EXISTS "index_lists_on_uid_and_updated_at" ON "lists" ("uid", "updated_at");
CREATE INDEX IF NOT EXISTS "index_lists_on_uid_and_deleted_at" ON "lists" ("uid", "deleted_at");
CREATE TABLE IF NOT EXISTS "tasks" ("id" varchar NOT NULL PRIMARY KEY, "uid" varchar NOT NULL, "title" varchar NOT NULL, "status" varchar DEFAULT 'active' NOT NULL, "position" float NOT NULL, "done_at" datetime(6), "archived_at" datetime(6), "deleted_at" datetime(6), "revision" integer DEFAULT 0 NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "list_id" varchar, "description" text);
CREATE INDEX IF NOT EXISTS "index_tasks_on_uid_and_status" ON "tasks" ("uid", "status");
CREATE INDEX IF NOT EXISTS "index_tasks_on_uid_and_updated_at" ON "tasks" ("uid", "updated_at");
CREATE INDEX IF NOT EXISTS "index_tasks_on_uid_and_deleted_at" ON "tasks" ("uid", "deleted_at");
CREATE INDEX IF NOT EXISTS "index_tasks_on_uid_and_list_id_and_status" ON "tasks" ("uid", "list_id", "status");
"#;

pub fn open(path: &str) -> rusqlite::Result<Connection> {
    if let Some(dir) = Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let conn = Connection::open(path)?;
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute_batch(SCHEMA)?;

    Ok(conn)
}

pub fn now() -> Ts {
    Utc::now().trunc_subsecs(6)
}

/// Storage format written by ActiveRecord's SQLite adapter (UTC, microseconds only when non-zero).
/// Keeping it identical keeps `since` string comparisons and a Rails rollback working.
pub fn to_db(t: &Ts) -> String {
    let format = if t.nanosecond() == 0 { "%Y-%m-%d %H:%M:%S" } else { "%Y-%m-%d %H:%M:%S%.6f" };
    t.format(format).to_string()
}

pub fn opt_to_db(t: &Option<Ts>) -> Option<String> {
    t.as_ref().map(to_db)
}

fn from_db(s: &str) -> Option<Ts> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").ok().map(|n| n.and_utc())
}

pub fn get_ts(row: &Row, idx: usize) -> rusqlite::Result<Option<Ts>> {
    let Some(raw) = row.get::<_, Option<String>>(idx)? else { return Ok(None) };

    from_db(&raw)
        .map(Some)
        .ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(idx, Type::Text, format!("invalid datetime {raw:?}").into()))
}

/// Lenient parsing of client-supplied timestamps; zone-less values are UTC, like `Time.zone.parse`.
pub fn parse_input(s: &str) -> Option<Ts> {
    let s = s.trim();

    if let Ok(t) = DateTime::parse_from_rfc3339(s) {
        return Some(t.with_timezone(&Utc).trunc_subsecs(6));
    }
    if let Ok(t) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f %z") {
        return Some(t.with_timezone(&Utc).trunc_subsecs(6));
    }
    for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"] {
        if let Ok(n) = NaiveDateTime::parse_from_str(s, format) {
            return Some(n.and_utc().trunc_subsecs(6));
        }
    }

    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok().map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

/// JSON format Rails emits for timestamps (ISO 8601, milliseconds, `Z`).
pub fn ser_ts<S: Serializer>(t: &Option<Ts>, s: S) -> Result<S::Ok, S::Error> {
    match t {
        Some(t) => s.serialize_str(&t.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
        None => s.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_format_round_trips() {
        let t = parse_input("2026-02-01T16:10:20.123456Z").unwrap();
        assert_eq!(to_db(&t), "2026-02-01 16:10:20.123456");
        assert_eq!(from_db(&to_db(&t)), Some(t));

        let whole = parse_input("2026-02-01T16:10:20Z").unwrap();
        assert_eq!(to_db(&whole), "2026-02-01 16:10:20");
        assert_eq!(from_db("2026-02-01 16:10:20"), Some(whole));
    }

    #[test]
    fn parses_client_inputs() {
        assert_eq!(to_db(&parse_input("2026-02-01T17:10:20.5+01:00").unwrap()), "2026-02-01 16:10:20.500000");
        assert_eq!(to_db(&parse_input("2026-02-01T16:10:20.1234567Z").unwrap()), "2026-02-01 16:10:20.123456");
        assert_eq!(to_db(&parse_input("2026-02-01").unwrap()), "2026-02-01 00:00:00");
        assert_eq!(parse_input(""), None);
        assert_eq!(parse_input("nope"), None);
    }
}
