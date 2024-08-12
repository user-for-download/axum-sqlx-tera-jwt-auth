use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::utils::date_config::date_format;
use chrono::serde::ts_seconds_option;
use sqlx::postgres::PgRow;
use sqlx::Row;
use validator_derive::Validate;

#[derive(Debug, Clone, Serialize)]
pub struct ListUser {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub img: Option<String>,
    pub is_verify: bool,
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
pub struct FormLogin {
    #[validate(email(message = "Email is not valid"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub purpose: String,
}

impl User {
    pub fn from_row(row: &PgRow) -> Self {
        User {
            id: row.get("id"),
            email: row.get("email"),
            username: row.get("username"),
            img: row.get("img"),
            is_verify: row.get("is_verify"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            password: row.get("password"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: i32,
    pub password: String,
    pub email: String,
    pub username: String,
    pub img: Option<String>,
    pub is_verify: bool,
    #[serde(with = "date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
}