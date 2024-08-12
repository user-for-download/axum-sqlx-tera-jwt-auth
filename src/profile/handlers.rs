use std::collections::HashMap;

use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse},
    extract::Query,
    response::Redirect,
};
use axum_extra::extract::Form;
use chrono::Utc;
use tera::Context;
use validator::Validate;

use crate::auth::models::User;
use crate::common::html_err;
use crate::common::Templates;
use crate::profile::models::{FormPasswordChange, FormVerifyEmail, PasswordChange, UpdateUserEmailVerify};
use crate::state::AppState;
use crate::utils::db::{check_email, get_user, query_update_password, query_update_user};
use crate::utils::jwt::{ar_hash_password, decode_token, encode_jwt};
use crate::utils::message::handle_errors;

pub async fn user(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut context = Context::new();

    let user = state.user.lock().map(|user| user.clone()).unwrap();

    match get_user(&state.db, user.email.clone()).await {
        Ok(row) => {
            context.insert("user", &User::from_row(&row));
            Ok(Html(templates.render("detail", &context).unwrap()).into_response())
        }
        Err(e) => Err(html_err(
            &templates,
            "detail",
            &mut context,
            format!("Error retrieving user: {:?}", e),
        ).await.into_response()),
    }
}

pub async fn get_verify_email(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let q_token = match params.get("token") {
        Some(token) => token.clone(),
        None => return Err(Redirect::to("/account/email-verify-resend").into_response()),
    };

    let user = match decode_token(q_token).await {
        Ok(claims) => claims.unwrap(),
        Err(_) => return Err(Redirect::to("/account/email-verify-resend").into_response()),
    };

    if user.purpose != "email-verify" {
        return Err(Redirect::to("/account/email-verify-resend").into_response());
    }

    let update = UpdateUserEmailVerify {
        email: user.email,
        is_verify: true,
        updated_at: Some(Utc::now()),
    };

    match query_update_user(&state.db, update).await {
        Ok(_) => Ok(Redirect::to("/account/login").into_response()),
        Err(_) => Err(Redirect::to("/account/email-verify-resend").into_response()),
    }
}

pub async fn get_verify_email_resend(
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    Html(templates.render("email-verify-resend", &Context::new()).unwrap()).into_response()
}

pub async fn post_verify_email_resend(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
    Form(form): Form<FormVerifyEmail>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut context = Context::new();

    if let Err(errors) = form.validate() {
        context.insert("messages", &handle_errors(errors).await);
        return Err(Html(templates.render("email-verify-resend", &context).unwrap()).into_response());
    }

    if let Ok(false) = check_email(&state.db, form.email.clone()).await {
        return Err(html_err(
            &templates,
            "email-verify-resend",
            &mut context,
            format!("Email not found: {:?}", form.email),
        ).await.into_response());
    }

    match encode_jwt(form.email.clone(), "email-verify".to_string(), 1).await {
        Ok(token) => {
            context.insert("verify", &token);
            Ok(Html(templates.render("email-verify", &context).unwrap()).into_response())
        }
        Err(e) => Err(html_err(
            &templates,
            "email-verify-resend",
            &mut context,
            format!("Error encoding JWT: {:?}", e),
        ).await.into_response()),
    }
}

pub async fn get_password_reset(
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    Html(templates.render("reset-password", &Context::new()).unwrap()).into_response()
}

pub async fn post_password_reset(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
    Form(form): Form<FormVerifyEmail>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut context = Context::new();

    if let Err(errors) = form.validate() {
        context.insert("messages", &handle_errors(errors).await);
        return Err(Html(templates.render("reset-password", &context).unwrap()).into_response());
    }

    if let Ok(false) = check_email(&state.db, form.email.clone()).await {
        return Err(html_err(
            &templates,
            "reset-password",
            &mut context,
            format!("Email not found: {:?}", form.email),
        ).await.into_response());
    }

    let user = match get_user(&state.db, form.email.clone()).await {
        Ok(row) => User::from_row(&row),
        Err(e) => return Err(html_err(
            &templates,
            "reset-password",
            &mut context,
            format!("Error retrieving user: {:?}", e),
        ).await.into_response()),
    };

    match encode_jwt(user.email.clone(), "reset-password".to_string(), 1).await {
        Ok(token) => {
            context.insert("pwd", &token);
            Ok(Html(templates.render("email-verify", &context).unwrap()).into_response())
        }
        Err(e) => Err(html_err(
            &templates,
            "reset-password",
            &mut context,
            format!("Error encoding JWT: {:?}", e),
        ).await.into_response()),
    }
}

pub async fn get_reset_password_confirm(
    Query(params): Query<HashMap<String, String>>,
    Extension(templates): Extension<Templates>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut context = Context::new();

    let q_token = match params.get("token") {
        Some(token) => token.clone(),
        None => return Err(Redirect::to("/account/reset-password").into_response()),
    };

    let user = match decode_token(q_token).await {
        Ok(claims) => claims.unwrap(),
        Err(_) => return Err(Redirect::to("/account/reset-password").into_response()),
    };

    if user.purpose == "reset-password" {
        Ok(Html(
            templates
                .render("reset-password-confirm", &Context::new())
                .unwrap()
        ).into_response())
    } else {
        let error_html = html_err(
            &templates,
            "reset-password-confirm",
            &mut context,
            "Token is not for password reset!".to_string(),
        ).await;

        Err(error_html.into_response())
    }
}

pub async fn post_reset_password_confirm(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    Extension(templates): Extension<Templates>,
    Form(form): Form<FormPasswordChange>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut context = Context::new();

    if let Err(errors) = form.validate() {
        context.insert("messages", &handle_errors(errors).await);
        return Err(Html(templates.render("reset-password-confirm", &context).unwrap()).into_response());
    }

    let q_token = match params.get("token") {
        Some(token) => token.clone(),
        None => return Err(Redirect::to("/account/reset-password").into_response()),
    };

    let user = match decode_token(q_token).await {
        Ok(claims) => claims.unwrap(),
        Err(_) => return Err(Redirect::to("/account/reset-password").into_response()),
    };

    if user.purpose != "reset-password" {
        return Err(Redirect::to("/account/reset-password").into_response());
    }

    let user = match get_user(&state.db, user.email.clone()).await {
        Ok(row) => User::from_row(&row),
        Err(e) => return Err(html_err(
            &templates,
            "reset-password-confirm",
            &mut context,
            format!("Error retrieving user: {:?}", e),
        ).await.into_response()),
    };

    let hashed_password = match ar_hash_password(&form.password) {
        Ok(hashed) => hashed,
        Err(e) => return Err(html_err(
            &templates,
            "reset-password-confirm",
            &mut context,
            format!("Error hashing password: {:?}", e),
        ).await.into_response()),
    };

    let password_change = PasswordChange {
        email: user.email,
        password: hashed_password,
        updated_at: Some(Utc::now()),
    };

    match query_update_password(&state.db, password_change).await {
        Ok(_) => Ok(Redirect::to("/account/login").into_response()),
        Err(e) => Err(html_err(
            &templates,
            "reset-password-confirm",
            &mut context,
            format!("Error updating password: {:?}", e),
        ).await.into_response()),
    }
}
