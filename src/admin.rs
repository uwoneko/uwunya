use std::sync::Arc;
use askama::Template;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Router;
use axum::routing::{get, post};
use axum_macros::debug_handler;
use crate::config::{MESSAGES, CONFIG, write_toml};
use axum_extra::headers::{authorization::Basic, Authorization, Header};
use serde::Deserialize;
use subtle::ConstantTimeEq;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(admin))
        .route("/set-likes", get(set_likes))
        .route("/set-working-on", get(set_working_on))
        .route("/motd/new", get(add_motd))
        .route("/motd/{id}/edit", get(edit_motd))
        .route("/motd/{id}/delete", get(delete_motd))
        .route("/motd/{id}/up", get(motd_up))
        .route("/motd/{id}/down", get(motd_down))
}

const UNAUTHORIZED_BASIC: (StatusCode, [(&str, &str); 1]) =
    (StatusCode::UNAUTHORIZED, [("WWW-Authenticate", "Basic realm=\"Restricted area\"")]);

macro_rules! basic_auth {
    ($headers: expr) => {
        let mut values = $headers.get_all("authorization").iter();
        let Ok(auth): Result<Authorization<Basic>, _> = Authorization::decode(&mut values) else {
            return UNAUTHORIZED_BASIC.into_response();
        };

        if !bool::from(auth.password().as_bytes().ct_eq(CONFIG.admin_password.as_bytes())) {
            return UNAUTHORIZED_BASIC.into_response();
        }
    };
}

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate {
    likes: Arc<str>,
    working_on: Arc<str>,
    motds: Vec<Arc<str>>
}

async fn admin(
    headers: HeaderMap
) -> Response {
    basic_auth!(headers);
    let messages = MESSAGES.read().await;

    askama_axum::IntoResponse::into_response(AdminTemplate {
        likes: messages.likes.clone(),
        working_on: messages.working_on.clone(),
        motds: messages.motds.clone()
    }).into_response()
}

#[derive(Deserialize)]
struct SetLikes {
    likes: Arc<str>
}

async fn set_likes(
    headers: HeaderMap,
    Query(set_likes): Query<SetLikes>
) -> Response {
    basic_auth!(headers);
    let mut messages = MESSAGES.write().await;

    messages.likes = set_likes.likes;
    if let Err(e) = write_toml("./messages.toml", &*messages) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("couldnt write into messages.toml: {e}")).into_response();
    }

    (StatusCode::SEE_OTHER, [("Location", "/admin")]).into_response()
}

#[derive(Deserialize)]
struct SetWorkingOn {
    working_on: Arc<str>
}

async fn set_working_on(
    headers: HeaderMap,
    Query(set_working_on): Query<SetWorkingOn>
) -> Response {
    basic_auth!(headers);
    let mut messages = MESSAGES.write().await;

    messages.working_on = set_working_on.working_on;
    if let Err(e) = write_toml("./messages.toml", &*messages) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("couldnt write into messages.toml: {e}")).into_response();
    }

    (StatusCode::SEE_OTHER, [("Location", "/admin")]).into_response()
}

#[derive(Deserialize)]
struct AddMotd {
    motd: Arc<str>
}

async fn add_motd(
    headers: HeaderMap,
    Query(add_motd): Query<AddMotd>
) -> Response {
    basic_auth!(headers);
    let mut messages = MESSAGES.write().await;

    messages.motds.push(add_motd.motd);
    if let Err(e) = write_toml("./messages.toml", &*messages) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("couldnt write into messages.toml: {e}")).into_response();
    }

    (StatusCode::SEE_OTHER, [("Location", "/admin")]).into_response()
}

#[derive(Deserialize)]
struct EditMotd {
    motd: Arc<str>
}

async fn edit_motd(
    headers: HeaderMap,
    Path(id): Path<usize>,
    Query(edit_motd): Query<EditMotd>
) -> Response {
    basic_auth!(headers);
    let mut messages = MESSAGES.write().await;

    messages.motds[id] = edit_motd.motd;
    if let Err(e) = write_toml("./messages.toml", &*messages) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("couldnt write into messages.toml: {e}")).into_response();
    }

    (StatusCode::SEE_OTHER, [("Location", "/admin")]).into_response()
}

async fn delete_motd(
    headers: HeaderMap,
    Path(id): Path<usize>,
) -> Response {
    basic_auth!(headers);
    let mut messages = MESSAGES.write().await;

    messages.motds.remove(id);
    if let Err(e) = write_toml("./messages.toml", &*messages) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("couldnt write into messages.toml: {e}")).into_response();
    }

    (StatusCode::SEE_OTHER, [("Location", "/admin")]).into_response()
}

async fn motd_up(
    headers: HeaderMap,
    Path(id): Path<usize>,
) -> Response {
    basic_auth!(headers);
    let mut messages = MESSAGES.write().await;

    messages.motds.swap(id, id + 1);
    if let Err(e) = write_toml("./messages.toml", &*messages) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("couldnt write into messages.toml: {e}")).into_response();
    }

    (StatusCode::SEE_OTHER, [("Location", "/admin")]).into_response()
}

async fn motd_down(
    headers: HeaderMap,
    Path(id): Path<usize>,
) -> Response {
    basic_auth!(headers);
    let mut messages = MESSAGES.write().await;

    messages.motds.swap(id, id - 1);
    if let Err(e) = write_toml("./messages.toml", &*messages) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("couldnt write into messages.toml: {e}")).into_response();
    }

    (StatusCode::SEE_OTHER, [("Location", "/admin")]).into_response()
}
