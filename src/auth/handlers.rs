use axum::{
    Extension,
    extract::{Form, State}
    ,
    response::{Html, IntoResponse},
};
use axum::response::Redirect;
use chrono::Utc;
use tera::Context;
use validator::Validate;

use crate::auth::models::{FormLogin, User};
use crate::common::{build_redirect_with_cookie, html_err, Templates};
use crate::profile::models::{FormSingUpUser, NewUser};
use crate::state::AppState;
use crate::utils::date_option::get_max_age_seconds;
use crate::utils::db::{check_email, check_username, get_user, query_new_user};
use crate::utils::jwt::{ar_hash_password, ar_verify_password, encode_jwt};
use crate::utils::message::handle_errors;

pub async fn get_signup(
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    Html(templates.render("signup", &Context::new()).unwrap())
}

pub async fn post_signup(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
    Form(form): Form<FormSingUpUser>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut context = Context::new();

    if let Err(errors) = form.validate() {
        context.insert("messages", &handle_errors(errors).await);
        return Err(Html(templates.render("signup", &context).unwrap()));
    }

    if let Ok(true) = check_email(&state.db, form.email.clone()).await {
        return Err(html_err(
            &templates,
            "signup",
            &mut context,
            format!("Email already exists: {:?}", form.email),
        ).await);
    }

    if let Ok(true) = check_username(&state.db, form.email.clone()).await {
        return Err(html_err(
            &templates,
            "signup",
            &mut context,
            format!("Username already exists: {:?}", form.username),
        ).await);
    }

    let hashed_password = match ar_hash_password(&form.password.clone()) {
        Ok(hashed) => hashed,
        Err(e) => return Err(html_err(&templates, "signup", &mut context, format!("Hashed Error: {:?}", e)).await)
    };

    let new_user = NewUser {
        email: form.email,
        username: form.username,
        password: hashed_password,
        is_verify: false,
        created_at: Utc::now(),
    };

    if let Err(e) = query_new_user(&state.db, new_user.clone()).await {
        return Err(html_err(
            &templates,
            "signup",
            &mut context,
            format!("Error Create: {:?}", e),
        ).await);
    }

    match encode_jwt(new_user.email.clone(), "email-verify".to_string(), 1).await {
        Ok(token) => {
            //  this need send email
            context.insert("verify", &token);
            Ok(Html(templates.render("email-verify", &context).unwrap()))
        }
        Err(e) => {
            return Err(html_err(
                &templates,
                "signup",
                &mut context,
                format!("Error encode jwt: {:?}", e),
            ).await);
        }
    }
}

pub async fn get_login(
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    Html(templates.render("login", &Context::new()).unwrap())
}

pub async fn post_login(
    State(state): State<AppState>,
    Extension(templates): Extension<Templates>,
    Form(form): Form<FormLogin>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let mut context = Context::new();

    if let Err(errors) = form.validate() {
        context.insert("messages", &handle_errors(errors).await);
        return Err(Html(templates.render("login", &context).unwrap()));
    }

    let user = match get_user(&state.db, form.email.clone()).await {
        Ok(row) => User::from_row(&row),
        Err(e) => return Err(html_err(
            &templates,
            "signup",
            &mut context,
            format!("Error Get User: {:?}", e),
        ).await)
    };

    match ar_verify_password(&form.password, &user.password) {
        Ok(_) => {}
        Err(e) => return Err(html_err(
            &templates,
            "signup",
            &mut context,
            format!("Error Verify Password: {:?}", e),
        ).await)
    }
    if !user.is_verify {
        return Ok(Redirect::to("/account/email-verify-resend").into_response());
    }
    let token = match encode_jwt(user.email.clone(), "auth".to_string(), 12).await {
        Ok(token) => token,
        Err(e) => {
            eprintln!("JWT creation failed: {:?}", e); // Log the error
            return Err(html_err(
                &templates,
                "login",
                &mut context,
                "An error occurred during login. Please try again.".to_string(),
            ).await);
        }
    };
    Ok(build_redirect_with_cookie(&token, get_max_age_seconds(), "/account/detail"))
}

pub async fn get_logout(
    Extension(templates): Extension<Templates>,
) -> impl IntoResponse {
    Html(templates.render("logout", &Context::new()).unwrap())
}


pub async fn post_logout(
) -> impl IntoResponse {
    build_redirect_with_cookie("", "0".to_string(), "/account/login")
}