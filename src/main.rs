use std::borrow::Cow;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use dotenv::dotenv;
use tokio::net::TcpListener;
use tokio::signal;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use axum_example::auth::middleware::cookie_to_state;
use axum_example::routes_account;
use axum_example::routes_assets;
use axum_example::routes_index;
use axum_example::state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();


    let state = match AppState::new().await {
        Ok(state) => state,
        Err(err) => {
            error!("Failed to create app state: {:?}", err);
            return;
        }
    };

    let assets_router = routes_assets::build_routes();

    let index_router = routes_index::build_routes(state.clone());
    let account_router = routes_account::build_routes(state.clone());

    let app = Router::new()
        .merge(assets_router)
        .merge(index_router)
        .merge(account_router)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(from_fn_with_state(state.clone(), cookie_to_state))
                .layer(TraceLayer::new_for_http())
        )
        .fallback(handler_404);

    let addr = SocketAddr::from((Ipv4Addr::new(127, 0, 0, 1), 8000));
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!("Failed to bind to address {}: {:?}", addr, err);
            return;
        }
    };

    info!("Listening on {}", addr);

    if let Err(err) = axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!("Server error: {:?}", err);
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404")
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }
    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {error}")),
    )
}