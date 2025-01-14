use std::sync::Arc;
use askama::Template;
use axum::{response::Html, routing::get, Router};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/public", ServeDir::new("./public"))
        .route("/", get(index));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

const MOTDS: [&str; 1] = ["meow :3"];

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    motd: &'static str
}

async fn index() -> Html<Box<str>> {
    let template = IndexTemplate {
        motd: MOTDS[0],
    };

    Html(template.render().unwrap().into())
}