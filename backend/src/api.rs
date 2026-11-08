use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use rusqlite::Connection;
use serde_json::{Map, Value, json};

use crate::db::{self, Ts};

pub type Db = std::sync::Arc<Mutex<Connection>>;
pub type Object = Map<String, Value>;

#[derive(Debug)]
pub enum ApiError {
    BadRequest,
    UidRequired,
    NotFound,
    /// A record failed validation or ownership checks; batch endpoints skip such items.
    Rejected(&'static str),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::UidRequired => (StatusCode::UNAUTHORIZED, json!({ "error": "uid_required" })),
            ApiError::BadRequest => status_body(StatusCode::BAD_REQUEST),
            ApiError::NotFound => status_body(StatusCode::NOT_FOUND),
            ApiError::Rejected(_) => status_body(StatusCode::UNPROCESSABLE_ENTITY),
            ApiError::Internal => status_body(StatusCode::INTERNAL_SERVER_ERROR),
        };

        (status, Json(body)).into_response()
    }
}

fn status_body(status: StatusCode) -> (StatusCode, Value) {
    (status, json!({ "status": status.as_u16(), "error": status.canonical_reason() }))
}

impl From<rusqlite::Error> for ApiError {
    fn from(error: rusqlite::Error) -> Self {
        eprintln!("database error: {error}");
        ApiError::Internal
    }
}

pub fn lock(db: &Db) -> MutexGuard<'_, Connection> {
    db.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn parse_body(body: &[u8]) -> Result<Option<Object>, ApiError> {
    if body.iter().all(u8::is_ascii_whitespace) {
        return Ok(None);
    }

    match serde_json::from_slice(body) {
        Ok(Value::Object(obj)) => Ok(Some(obj)),
        _ => Err(ApiError::BadRequest),
    }
}

/// `X-UID` header, falling back to a `uid` param (body wins over query, like Rails params).
pub fn current_uid(headers: &HeaderMap, query: &HashMap<String, String>, body: Option<&Object>) -> Result<String, ApiError> {
    let uid = match headers.get("x-uid") {
        Some(value) => value.to_str().ok().map(str::to_owned),
        None => body
            .and_then(|b| b.get("uid"))
            .and_then(scalar_string)
            .or_else(|| query.get("uid").cloned()),
    };

    uid.filter(|u| !is_blank(u)).ok_or(ApiError::UidRequired)
}

/// `params[:since]`; an unparseable value disables the filter, as before.
pub fn since(query: &HashMap<String, String>) -> Option<String> {
    query.get("since").and_then(|s| db::parse_input(s)).map(|t| db::to_db(&t))
}

/// `params.require(key)` for a batch: a non-empty array.
pub fn require_items<'a>(body: Option<&'a Object>, key: &str) -> Result<&'a [Value], ApiError> {
    match body.and_then(|b| b.get(key)) {
        Some(Value::Array(items)) if !items.is_empty() => Ok(items),
        _ => Err(ApiError::BadRequest),
    }
}

/// A batch item as an object with an `id`.
pub fn batch_item(item: &Value) -> Result<(&Object, String), ApiError> {
    let attrs = item.as_object().ok_or(ApiError::Rejected("item is not an object"))?;
    let mut id = None;
    assign_string(attrs, "id", &mut id);

    Ok((attrs, id.ok_or(ApiError::Rejected("missing id"))?))
}

/// Logs a skipped batch item, so one bad outbox entry never blocks a client's sync.
pub fn log_skipped(endpoint: &str, item: &Value, reason: &str) {
    eprintln!("{endpoint}: skipped item {}: {reason}", item.get("id").unwrap_or(&Value::Null));
}

pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

pub fn present(s: &Option<String>) -> bool {
    s.as_deref().is_some_and(|s| !is_blank(s))
}

fn scalar_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "t" } else { "f" }.to_owned()),
        _ => None,
    }
}

// Attribute assignment mirrors strong params + ActiveRecord casting: absent keys and
// non-scalar values leave the field untouched, `null` clears it.

pub fn assign_string(attrs: &Object, key: &str, field: &mut Option<String>) {
    match attrs.get(key) {
        Some(Value::Null) => *field = None,
        Some(value) => {
            if let Some(s) = scalar_string(value) {
                *field = Some(s);
            }
        }
        None => {}
    }
}

pub fn assign_time(attrs: &Object, key: &str, field: &mut Option<Ts>) {
    match attrs.get(key) {
        Some(Value::String(s)) => *field = db::parse_input(s),
        Some(Value::Null | Value::Number(_) | Value::Bool(_)) => *field = None,
        _ => {}
    }
}

/// Fails like the numericality validation for non-numeric input.
pub fn assign_float(attrs: &Object, key: &str, field: &mut Option<f64>) -> Result<(), ApiError> {
    match attrs.get(key) {
        Some(Value::Null) => *field = None,
        Some(Value::Number(n)) => *field = n.as_f64(),
        Some(Value::String(s)) if is_blank(s) => *field = None,
        Some(Value::String(s)) => match s.trim().parse::<f64>() {
            Ok(v) if v.is_finite() => *field = Some(v),
            _ => return Err(ApiError::Rejected("position is not a number")),
        },
        Some(Value::Bool(_)) => return Err(ApiError::Rejected("position is not a number")),
        _ => {}
    }

    Ok(())
}
