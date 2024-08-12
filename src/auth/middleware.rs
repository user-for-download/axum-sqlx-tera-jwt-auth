use axum::extract::{Request, State};
use axum::http::header::COOKIE;
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use tracing::{error, info};
use crate::auth::models::User;
use crate::common::AuthenticatedUser;
use crate::state::AppState;
use crate::utils::cookie::extract_cookie_value;
use crate::utils::db::get_user;
use crate::utils::jwt::decode_token;

async fn clear_current_user(state: &AppState) {
    if let Ok(mut user) = state.user.lock() {
        *user = AuthenticatedUser::default();
    } else {
        error!("Failed to acquire lock to clear current user.");
    }
}

async fn update_current_user(state: &AppState, claims: AuthenticatedUser) {
    if let Ok(mut user) = state.user.lock() {
        *user = claims;
    } else {
        error!("Failed to acquire lock to update current user.");
    }
}

pub async fn cookie_to_state(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Clear the current user before processing the request
    clear_current_user(&state).await;

    // Get the COOKIE header from the request
    let cookie_header = request
        .headers()
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    // Extract the "visit" token from the cookie
    let token = match extract_cookie_value(cookie_header, "visit") {
        Some(token) => token,
        None => return next.run(request).await, // Return the response if no token is found
    };

    // Decode the token to extract user claims
    let claims = match decode_token(token).await {
        Ok(Some(claims)) => claims,
        Ok(None) | Err(_) => return next.run(request).await, // Return the response if decoding fails
    };

    // Fetch the user details based on the claims
    match get_user(&state.db, claims.email.clone()).await {
        Ok(row) => {
            let user = User::from_row(&row);
            info!("User details found: {:?}", user);
            update_current_user(&state, AuthenticatedUser {
                email: user.email,
                purpose: "".to_string(),
            }).await;
        }
        Err(_) => {
            info!("User details not found for email: {:?}", claims.email);
        }
    };

    next.run(request).await
}

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    match state.user.lock() {
        Ok(user) => {
            if user.email.is_empty() {
                info!("User is not authenticated. Redirecting to login page.");
                return Redirect::to("/account/login").into_response();
            }
        }
        Err(_) => {
            info!("Failed to acquire lock to check current user.");
            return Redirect::to("/account/login").into_response();
        }
    }
    next.run(request).await
}

pub async fn require_guest(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Attempt to acquire the lock to check the user state
    let is_authenticated = match state.user.lock() {
        Ok(user) => !user.email.is_empty(), // User is authenticated if email is not empty
        Err(_) => {
            info!("Failed to acquire lock to check current user.");
            // Handle lock acquisition failure by redirecting to logout page
            return Redirect::to("/account/logout").into_response();
        }
    };
    // If the user is authenticated, redirect to the logout page
    if is_authenticated {
        info!("User is authenticated. Redirecting to logout.");
        Redirect::to("/account/logout").into_response()
    } else {
        // Otherwise, proceed with the request
        next.run(request).await
    }
}