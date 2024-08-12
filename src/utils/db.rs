use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool, postgres::PgRow};

use crate::profile::models::{NewUser, PasswordChange, UpdateUserEmailVerify};

/// Enum representing various database query errors.
#[derive(Debug)]
pub enum QueryError {
    Duplicate,
    NotFound,
    DatabaseError(SqlxError),
    RowNotFound,
    Query,
    TypeNotFound,
    ConnectionClosed,
    PoolTimedOut,
    Tls,
    Protocol,
    InvalidQuery,
    Io,
}


impl From<SqlxError> for QueryError {
    fn from(err: SqlxError) -> Self {
        match err {
            SqlxError::RowNotFound => QueryError::RowNotFound,
            SqlxError::PoolTimedOut => QueryError::PoolTimedOut,
            _ => QueryError::DatabaseError(err),
        }
    }
}

/// Trait defining database utility methods.
#[async_trait]
pub trait DatabaseUtils {
    async fn find_record(&self, query: &str, param: &str) -> Result<bool, QueryError>;
    async fn select_existence(&self, query: &str, param: &str) -> Result<PgRow, QueryError>;
}

#[async_trait]
impl DatabaseUtils for PgPool {
    async fn find_record(&self, query: &str, param: &str) -> Result<bool, QueryError> {
        let exists: (bool,) = sqlx::query_as(query)
            .bind(param)
            .fetch_one(self)
            .await
            .map_err(QueryError::from)?;
        Ok(exists.0)
    }

    async fn select_existence(&self, query: &str, param: &str) -> Result<PgRow, QueryError> {
        let row = sqlx::query(query)
            .bind(param)
            .fetch_one(self)
            .await
            .map_err(QueryError::from)?;
        Ok(row)
    }
}

/// Check if the email exists in the database.
pub async fn check_email(state: &PgPool, email: String) -> Result<bool, QueryError> {
    let query = "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)";
    state.find_record(query, &email).await
}

/// Check if the username exists in the database.
pub async fn check_username(state: &PgPool, username: String) -> Result<bool, QueryError> {
    let query = "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)";
    state.find_record(query, &username).await
}

/// Retrieve a user by email from the database.
pub async fn get_user(state: &PgPool, email: String) -> Result<PgRow, QueryError> {
    let query = "SELECT * FROM users WHERE email = $1";
    state.select_existence(query, &email).await
}

/// Insert a new user into the database.
pub async fn query_new_user(state: &PgPool, user: NewUser) -> Result<(), QueryError> {
    let query = "
        INSERT INTO users (email, username, password, is_verify, created_at)
        VALUES ($1, $2, $3, $4, $5)
    ";
    sqlx::query(query)
        .bind(&user.email)
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.is_verify)
        .bind(user.created_at)
        .execute(state)
        .await
        .map_err(QueryError::from)?;
    Ok(())
}

/// Update a user's email verification status.
pub async fn query_update_user(state: &PgPool, user: UpdateUserEmailVerify) -> Result<(), QueryError> {
    let query = "UPDATE users SET is_verify = $2, updated_at = $3 WHERE email = $1";
    sqlx::query(query)
        .bind(&user.email)
        .bind(user.is_verify)
        .bind(user.updated_at)
        .execute(state)
        .await
        .map_err(QueryError::from)?;
    Ok(())
}

/// Update a user's password.
pub async fn query_update_password(state: &PgPool, user: PasswordChange) -> Result<(), QueryError> {
    let query = "UPDATE users SET password = $2, updated_at = $3 WHERE email = $1";
    sqlx::query(query)
        .bind(&user.email)
        .bind(&user.password)
        .bind(user.updated_at)
        .execute(state)
        .await
        .map_err(QueryError::from)?;
    Ok(())
}
