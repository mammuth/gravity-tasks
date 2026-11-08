use std::collections::HashMap;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::api::{self, ApiError, Db, Object};
use crate::db::{self, Ts};
use crate::lists;

const COLUMNS: &str =
    "id, uid, title, status, position, done_at, archived_at, deleted_at, revision, created_at, updated_at, list_id, description";
const PERMITTED: &[&str] =
    &["title", "description", "status", "position", "done_at", "archived_at", "deleted_at", "list_id"];
const STATUSES: &[&str] = &["active", "done", "archived"];

#[derive(Serialize)]
pub struct Task {
    pub id: Option<String>,
    pub uid: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub position: Option<f64>,
    #[serde(serialize_with = "db::ser_ts")]
    pub done_at: Option<Ts>,
    #[serde(serialize_with = "db::ser_ts")]
    pub archived_at: Option<Ts>,
    #[serde(serialize_with = "db::ser_ts")]
    pub deleted_at: Option<Ts>,
    pub revision: i64,
    #[serde(serialize_with = "db::ser_ts")]
    pub created_at: Option<Ts>,
    #[serde(serialize_with = "db::ser_ts")]
    pub updated_at: Option<Ts>,
    pub list_id: Option<String>,
    pub description: Option<String>,
}

impl Task {
    fn new(id: String, uid: &str) -> Self {
        Task {
            id: Some(id),
            uid: uid.to_owned(),
            title: None,
            status: Some("active".to_owned()),
            position: None,
            done_at: None,
            archived_at: None,
            deleted_at: None,
            revision: 0,
            created_at: None,
            updated_at: None,
            list_id: None,
            description: None,
        }
    }

    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Task {
            id: row.get(0)?,
            uid: row.get(1)?,
            title: row.get(2)?,
            status: row.get(3)?,
            position: row.get(4)?,
            done_at: db::get_ts(row, 5)?,
            archived_at: db::get_ts(row, 6)?,
            deleted_at: db::get_ts(row, 7)?,
            revision: row.get(8)?,
            created_at: db::get_ts(row, 9)?,
            updated_at: db::get_ts(row, 10)?,
            list_id: row.get(11)?,
            description: row.get(12)?,
        })
    }

    fn assign(&mut self, attrs: &Object) -> Result<(), ApiError> {
        api::assign_string(attrs, "title", &mut self.title);
        api::assign_string(attrs, "description", &mut self.description);
        api::assign_string(attrs, "status", &mut self.status);
        api::assign_float(attrs, "position", &mut self.position)?;
        api::assign_time(attrs, "done_at", &mut self.done_at);
        api::assign_time(attrs, "archived_at", &mut self.archived_at);
        api::assign_time(attrs, "deleted_at", &mut self.deleted_at);
        api::assign_string(attrs, "list_id", &mut self.list_id);
        Ok(())
    }

    /// Validates and persists, setting timestamps like ActiveRecord does on save.
    fn save(&mut self, conn: &Connection, persisted: bool) -> Result<(), ApiError> {
        let valid_status = self.status.as_deref().is_some_and(|s| STATUSES.contains(&s));
        if api::is_blank(&self.uid)
            || !api::present(&self.title)
            || !valid_status
            || self.position.is_none()
            || !api::present(&self.list_id)
        {
            return Err(ApiError::Invalid);
        }

        let now = db::now();
        self.updated_at = Some(now);
        if !persisted {
            self.created_at = Some(now);
        }

        let sql = if persisted {
            "UPDATE tasks SET uid = ?2, title = ?3, status = ?4, position = ?5, done_at = ?6, archived_at = ?7, deleted_at = ?8, \
             revision = ?9, created_at = ?10, updated_at = ?11, list_id = ?12, description = ?13 WHERE id = ?1"
        } else {
            "INSERT INTO tasks (id, uid, title, status, position, done_at, archived_at, deleted_at, revision, created_at, updated_at, list_id, description) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
        };
        conn.execute(
            sql,
            params![
                self.id,
                self.uid,
                self.title,
                self.status,
                self.position,
                db::opt_to_db(&self.done_at),
                db::opt_to_db(&self.archived_at),
                db::opt_to_db(&self.deleted_at),
                self.revision,
                db::opt_to_db(&self.created_at),
                db::opt_to_db(&self.updated_at),
                self.list_id,
                self.description,
            ],
        )?;

        Ok(())
    }
}

fn find(conn: &Connection, id: &str) -> rusqlite::Result<Option<Task>> {
    conn.query_row(&format!("SELECT {COLUMNS} FROM tasks WHERE id = ?1 LIMIT 1"), [id], Task::from_row)
        .optional()
}

pub async fn index(
    State(db): State<Db>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let uid = api::current_uid(&headers, &query, None)?;
    let conn = api::lock(&db);

    let tasks = match api::since(&query) {
        Some(since) => {
            let sql = format!("SELECT {COLUMNS} FROM tasks WHERE uid = ?1 AND (updated_at > ?2 OR deleted_at > ?2)");
            conn.prepare(&sql)?.query_map(params![uid, since], Task::from_row)?.collect::<Result<_, _>>()?
        }
        None => {
            let sql = format!("SELECT {COLUMNS} FROM tasks WHERE uid = ?1");
            conn.prepare(&sql)?.query_map([uid], Task::from_row)?.collect::<Result<_, _>>()?
        }
    };

    Ok(Json(tasks))
}

pub async fn create(
    State(db): State<Db>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    let body = api::parse_body(&body)?;
    let uid = api::current_uid(&headers, &query, body.as_ref())?;
    let attrs = api::require_record(body.as_ref(), "task", PERMITTED)?;

    let mut conn = api::lock(&db);
    let tx = conn.transaction()?;
    let mut task = Task::new(uuid::Uuid::new_v4().to_string(), &uid);
    task.assign(attrs)?;
    if !api::present(&task.list_id) {
        task.list_id = Some(lists::ensure_inbox(&tx, &uid)?);
    }
    task.save(&tx, false)?;
    tx.commit()?;

    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn update(
    State(db): State<Db>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Json<Task>, ApiError> {
    let body = api::parse_body(&body)?;
    let uid = api::current_uid(&headers, &query, body.as_ref())?;
    let attrs = api::require_record(body.as_ref(), "task", PERMITTED)?;

    let conn = api::lock(&db);
    let mut task = find(&conn, &id)?.filter(|t| t.uid == uid).ok_or(ApiError::NotFound)?;
    task.assign(attrs)?;
    task.revision += 1;
    task.save(&conn, true)?;

    Ok(Json(task))
}

/// Upserts the client's outbox. Runs in one transaction, so a rejected item leaves nothing half-applied.
pub async fn batch(
    State(db): State<Db>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Json<Vec<Task>>, ApiError> {
    let body = api::parse_body(&body)?;
    let uid = api::current_uid(&headers, &query, body.as_ref())?;
    let items = api::require_items(body.as_ref(), "tasks")?;

    let mut conn = api::lock(&db);
    let tx = conn.transaction()?;
    let mut inbox_id = None;
    let mut results = Vec::with_capacity(items.len());

    for attrs in items {
        let mut id = None;
        api::assign_string(attrs, "id", &mut id);
        let id = id.ok_or(ApiError::Invalid)?;

        let existing = find(&tx, &id)?;
        if existing.as_ref().is_some_and(|t| t.uid != uid) {
            return Err(ApiError::Conflict("task_id_conflict"));
        }

        let persisted = existing.is_some();
        let mut task = existing.unwrap_or_else(|| Task::new(id, &uid));
        task.assign(attrs)?;
        // As before: an item without a list_id (even for an existing task) goes to the inbox.
        let mut list_id = None;
        api::assign_string(attrs, "list_id", &mut list_id);
        if !api::present(&list_id) {
            if inbox_id.is_none() {
                inbox_id = Some(lists::ensure_inbox(&tx, &uid)?);
            }
            task.list_id = inbox_id.clone();
        }
        if persisted {
            task.revision += 1;
        }
        task.save(&tx, persisted)?;

        results.push(task);
    }

    tx.commit()?;
    Ok(Json(results))
}
