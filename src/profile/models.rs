use axum::{body::Body, http::Response};
use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

use crate::utils::date_config::date_format;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NaUser {
    pub username: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmUser {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password: String,
    pub is_verify: bool,
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListUser {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub img: Option<String>,
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormNewUser {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateUser {
    pub email: String,
    pub username: String,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormUpdateUser {
    pub email: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasswordChange {
    pub email: String,
    pub password: String,
    #[serde(with = "ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
pub struct FormPasswordChange {
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

pub enum EnumError {
    ResBody(Response<Body>),
    ErrString(String),
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
pub struct FormSingUpUser {
    #[validate(email(message = "Email is not valid"))]
    pub(crate) email: String,

    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub(crate) username: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub(crate) password: String,
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
pub struct GetEmailVerify {
    pub(crate) token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateUserEmailVerify {
    pub is_verify: bool,
    pub updated_at: Option<DateTime<Utc>>,
    pub email: String,
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
pub struct FormVerifyEmail {
    #[validate(email(message = "Email is not valid"))]
    pub(crate) email: String,
}
