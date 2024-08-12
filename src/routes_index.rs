use std::sync::Arc;

use axum::{Extension, Router, routing::get};
use axum::response::{Html, IntoResponse};
use headers::HeaderMap;
use tera::{Context, Tera};
use tracing::info;

use crate::common::{Templates, Timing};
use crate::state::AppState;

pub fn build_routes(state: AppState) -> Router {
    let mut base_tera = Tera::default();
    base_tera
        .add_raw_templates(vec![
            ("base.html", include_str!("../templates/base.html")),
            ("navbar.html", include_str!("../templates/navbar.html")),
            ("footer.html", include_str!("../templates/footer.html")),
            ("messages.html", include_str!("../templates/messages.html")),
            ("index", include_str!("../templates/index.html")),
        ])
        .unwrap();

    let index_routes = Router::new().nest(
        "/",
        Router::new()
            .route("/", get(index))
            .layer(Extension(Arc::new(base_tera))),
    );
    Router::new().nest("/", index_routes.with_state(state))
}

pub async fn index(
    time: Timing<HeaderMap>,
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    info!("index state: {:?}", time.duration);
    Html(templates.render("index", &Context::new()).unwrap())
}