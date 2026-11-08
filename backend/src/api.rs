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
    Conflict(&'static str),
    Invalid,
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::UidRequired => (StatusCode::UNAUTHORIZED, json!({ "error": "uid_required" })),
            ApiError::Conflict(error) => (StatusCode::CONFLICT, json!({ "error": error })),
            ApiError::BadRequest => status_body(StatusCode::BAD_REQUEST),
            ApiError::NotFound => status_body(StatusCode::NOT_FOUND),
            ApiError::Invalid => status_body(StatusCode::UNPROCESSABLE_ENTITY),
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

/// `params.require(key)` for a batch: a non-empty array of objects.
pub fn require_items<'a>(body: Option<&'a Object>, key: &str) -> Result<Vec<&'a Object>, ApiError> {
    match body.and_then(|b| b.get(key)) {
        Some(Value::Array(items)) if !items.is_empty() => {
            items.iter().map(|item| item.as_object().ok_or(ApiError::BadRequest)).collect()
        }
        _ => Err(ApiError::BadRequest),
    }
}

/// `params.require(key)` for single-record endpoints, including Rails' JSON params wrapping
/// (top-level attributes are accepted when the `key` wrapper is missing).
pub fn require_record<'a>(body: Option<&'a Object>, key: &str, permitted: &[&str]) -> Result<&'a Object, ApiError> {
    let body = body.ok_or(ApiError::BadRequest)?;

    match body.get(key) {
        Some(Value::Object(record)) if !record.is_empty() => Ok(record),
        Some(_) => Err(ApiError::BadRequest),
        None if permitted.iter().any(|k| body.contains_key(*k)) => Ok(body),
        None => Err(ApiError::BadRequest),
    }
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
            _ => return Err(ApiError::Invalid),
        },
        Some(Value::Bool(_)) => return Err(ApiError::Invalid),
        _ => {}
    }

    Ok(())
}
