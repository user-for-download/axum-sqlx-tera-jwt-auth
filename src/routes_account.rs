use std::sync::Arc;

use axum::{Extension, Router, routing::get};
use axum::middleware::from_fn_with_state;
use tera::Tera;
use tracing::log::error;

use crate::{auth, profile};
use crate::auth::middleware::{require_auth, require_guest};
use crate::state::AppState;

pub fn build_routes(state: AppState) -> Router {
    let mut user_tera = Tera::default();

    // Add raw templates to Tera
    if let Err(e) = user_tera.add_raw_templates(vec![
        ("base.html", include_str!("../templates/base.html")),
        ("navbar.html", include_str!("../templates/navbar.html")),
        ("footer.html", include_str!("../templates/footer.html")),
        ("messages.html", include_str!("../templates/messages.html")),
        ("login", include_str!("../templates/auth/login.html")),
        ("logout", include_str!("../templates/auth/logout.html")),
        ("signup", include_str!("../templates/auth/signup.html")),
        ("detail", include_str!("../templates/profile/detail.html")),
        ("update", include_str!("../templates/profile/update.html")),
        ("password_change", include_str!("../templates/profile/password-change.html")),
        ("email-verify-resend", include_str!("../templates/auth/email-verify-resend.html")),
        ("email-verify", include_str!("../templates/auth/email-verify.html")),
        ("reset-password", include_str!("../templates/auth/reset-password.html")),
        ("reset-password-confirm", include_str!("../templates/auth/reset-password-confirm.html")),
    ]) {
        error!("Error loading Tera templates: {}", e);
    }
    let auth_routes = Router::new().nest(
        "/",
        Router::new()
            .route("/detail", get(profile::handlers::user))
            .route(
                "/logout",
                get(auth::handlers::get_logout).post(auth::handlers::post_logout),
            )
            .layer(from_fn_with_state(state.clone(), require_auth)),
    );

    let guest_routes = Router::new().nest(
        "/",
        Router::new()
            .route(
                "/signup",
                get(auth::handlers::get_signup)
                    .post(auth::handlers::post_signup),
            )
            .route(
                "/login",
                get(auth::handlers::get_login)
                    .post(auth::handlers::post_login),
            )
            .route(
                "/email-verify",
                get(profile::handlers::get_verify_email),
            )
            .route(
                "/email-verify-resend",
                get(profile::handlers::get_verify_email_resend)
                    .post(profile::handlers::post_verify_email_resend),
            )
            .route(
                "/reset-password",
                get(profile::handlers::get_password_reset)
                    .post(profile::handlers::post_password_reset),
            )
            .route(
                "/reset-password-confirm",
                get(profile::handlers::get_reset_password_confirm)
                    .post(profile::handlers::post_reset_password_confirm),
            )
            .layer(from_fn_with_state(state.clone(), require_guest)),
    );
    Router::new().nest(
        "/account",
        Router::new()
            .nest("/", auth_routes)
            .nest("/", guest_routes)
            .layer(Extension(Arc::new(user_tera)))
            .with_state(state.clone()),
    )
}
