use std::collections::HashMap;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::api::{self, ApiError, Db, Object};
use crate::db::{self, Ts};

const COLUMNS: &str = "id, uid, name, position, deleted_at, revision, created_at, updated_at";
const PERMITTED: &[&str] = &["name", "position", "deleted_at"];

#[derive(Serialize)]
pub struct List {
    pub id: Option<String>,
    pub uid: String,
    pub name: Option<String>,
    pub position: Option<f64>,
    #[serde(serialize_with = "db::ser_ts")]
    pub deleted_at: Option<Ts>,
    pub revision: i64,
    #[serde(serialize_with = "db::ser_ts")]
    pub created_at: Option<Ts>,
    #[serde(serialize_with = "db::ser_ts")]
    pub updated_at: Option<Ts>,
}

impl List {
    pub fn new(id: String, uid: &str) -> Self {
        List {
            id: Some(id),
            uid: uid.to_owned(),
            name: None,
            position: None,
            deleted_at: None,
            revision: 0,
            created_at: None,
            updated_at: None,
        }
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(List {
            id: row.get(0)?,
            uid: row.get(1)?,
            name: row.get(2)?,
            position: row.get(3)?,
            deleted_at: db::get_ts(row, 4)?,
            revision: row.get(5)?,
            created_at: db::get_ts(row, 6)?,
            updated_at: db::get_ts(row, 7)?,
        })
    }

    fn assign(&mut self, attrs: &Object) -> Result<(), ApiError> {
        api::assign_string(attrs, "name", &mut self.name);
        api::assign_float(attrs, "position", &mut self.position)?;
        api::assign_time(attrs, "deleted_at", &mut self.deleted_at);
        Ok(())
    }

    /// Validates and persists, setting timestamps like ActiveRecord does on save.
    pub fn save(&mut self, conn: &Connection, persisted: bool) -> Result<(), ApiError> {
        if api::is_blank(&self.uid) || !api::present(&self.name) || self.position.is_none() {
            return Err(ApiError::Invalid);
        }

        let now = db::now();
        self.updated_at = Some(now);
        if !persisted {
            self.created_at = Some(now);
        }

        let sql = if persisted {
            "UPDATE lists SET uid = ?2, name = ?3, position = ?4, deleted_at = ?5, revision = ?6, created_at = ?7, updated_at = ?8 WHERE id = ?1"
        } else {
            "INSERT INTO lists (id, uid, name, position, deleted_at, revision, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        };
        conn.execute(
            sql,
            params![
                self.id,
                self.uid,
                self.name,
                self.position,
                db::opt_to_db(&self.deleted_at),
                self.revision,
                db::opt_to_db(&self.created_at),
                db::opt_to_db(&self.updated_at),
            ],
        )?;

        Ok(())
    }

    /// Soft-deleting a list soft-deletes all of its tasks.
    fn cascade_delete_tasks(&self, conn: &Connection) -> Result<(), ApiError> {
        if let Some(deleted_at) = &self.deleted_at {
            conn.execute(
                "UPDATE tasks SET deleted_at = ?1, updated_at = ?2 WHERE uid = ?3 AND list_id = ?4",
                params![db::to_db(deleted_at), db::to_db(&db::now()), self.uid, self.id],
            )?;
        }

        Ok(())
    }
}

/// The per-user default list that tasks without a `list_id` land in.
pub fn ensure_inbox(conn: &Connection, uid: &str) -> Result<String, ApiError> {
    let id = format!("inbox-{uid}");
    let exists = conn
        .query_row("SELECT 1 FROM lists WHERE id = ?1 AND uid = ?2 LIMIT 1", [&id, uid], |_| Ok(()))
        .optional()?
        .is_some();

    if !exists {
        let mut inbox = List::new(id.clone(), uid);
        inbox.name = Some("Inbox".to_owned());
        inbox.position = Some(1000.0);
        inbox.save(conn, false)?;
    }

    Ok(id)
}

fn find(conn: &Connection, id: &str) -> rusqlite::Result<Option<List>> {
    conn.query_row(&format!("SELECT {COLUMNS} FROM lists WHERE id = ?1 LIMIT 1"), [id], List::from_row)
        .optional()
}

pub async fn index(
    State(db): State<Db>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Vec<List>>, ApiError> {
    let uid = api::current_uid(&headers, &query, None)?;
    let conn = api::lock(&db);

    let lists = match api::since(&query) {
        Some(since) => {
            let sql = format!("SELECT {COLUMNS} FROM lists WHERE uid = ?1 AND (updated_at > ?2 OR deleted_at > ?2)");
            conn.prepare(&sql)?.query_map(params![uid, since], List::from_row)?.collect::<Result<_, _>>()?
        }
        None => {
            let sql = format!("SELECT {COLUMNS} FROM lists WHERE uid = ?1");
            conn.prepare(&sql)?.query_map([uid], List::from_row)?.collect::<Result<_, _>>()?
        }
    };

    Ok(Json(lists))
}

pub async fn create(
    State(db): State<Db>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<(StatusCode, Json<List>), ApiError> {
    let body = api::parse_body(&body)?;
    let uid = api::current_uid(&headers, &query, body.as_ref())?;
    let attrs = api::require_record(body.as_ref(), "list", PERMITTED)?;

    let mut list = List::new(uuid::Uuid::new_v4().to_string(), &uid);
    list.assign(attrs)?;
    list.save(&api::lock(&db), false)?;

    Ok((StatusCode::CREATED, Json(list)))
}

pub async fn update(
    State(db): State<Db>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Json<List>, ApiError> {
    let body = api::parse_body(&body)?;
    let uid = api::current_uid(&headers, &query, body.as_ref())?;
    let attrs = api::require_record(body.as_ref(), "list", PERMITTED)?;

    let mut conn = api::lock(&db);
    let tx = conn.transaction()?;
    let mut list = find(&tx, &id)?.filter(|l| l.uid == uid).ok_or(ApiError::NotFound)?;
    list.assign(attrs)?;
    list.revision += 1;
    list.save(&tx, true)?;
    list.cascade_delete_tasks(&tx)?;
    tx.commit()?;

    Ok(Json(list))
}

/// Upserts the client's outbox. Runs in one transaction, so a rejected item leaves nothing half-applied.
pub async fn batch(
    State(db): State<Db>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Json<Vec<List>>, ApiError> {
    let body = api::parse_body(&body)?;
    let uid = api::current_uid(&headers, &query, body.as_ref())?;
    let items = api::require_items(body.as_ref(), "lists")?;

    let mut conn = api::lock(&db);
    let tx = conn.transaction()?;
    let mut results = Vec::with_capacity(items.len());

    for attrs in items {
        let mut id = None;
        api::assign_string(attrs, "id", &mut id);
        let id = id.ok_or(ApiError::Invalid)?;

        let existing = find(&tx, &id)?;
        if existing.as_ref().is_some_and(|l| l.uid != uid) {
            return Err(ApiError::Conflict("list_id_conflict"));
        }

        let persisted = existing.is_some();
        let mut list = existing.unwrap_or_else(|| List::new(id, &uid));
        list.assign(attrs)?;
        if persisted {
            list.revision += 1;
        }
        list.save(&tx, persisted)?;
        list.cascade_delete_tasks(&tx)?;

        results.push(list);
    }

    tx.commit()?;
    Ok(Json(results))
}
