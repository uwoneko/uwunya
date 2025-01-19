mod config;
mod admin;

use axum_macros::debug_handler;
use std::convert::Into;
use std::sync::{Arc, LazyLock};
use askama_axum::{Template};
use axum::{response::Html, routing::get, Extension, Router};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::headers::{Authorization, Header};
use o2o::o2o;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tower_http::services::ServeDir;
use crate::config::{parse_toml, start_watching, Messages, CONFIG, MESSAGES};

#[tokio::main]
async fn main() {
    start_watching();
    
    let app = Router::new()
        .nest_service("/public", ServeDir::new("./public"))
        .route("/", get(index))
        .nest("/admin", admin::routes());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    motd: Arc<str>,
    likes: Arc<str>,
    working_on: Arc<str>
}

async fn index() -> Html<Box<str>> {
    let messages = MESSAGES.read().await;
    let template = IndexTemplate {
        motd: messages.motds[0].clone(),
        likes: messages.likes.clone(),
        working_on: messages.working_on.clone(),
    };

    Html(template.render().unwrap().into())
}
