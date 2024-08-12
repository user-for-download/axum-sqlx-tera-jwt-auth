use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use sqlx::PgPool;
use tracing::error;

use crate::common::AuthenticatedUser;

static DATABASE_URL: Lazy<String> = Lazy::new(|| {
    dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set")
});

impl AppState {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(&DATABASE_URL).await.map_err(|err| {
            error!("Failed to connect to the database: {:?}", err);
            err
        })?;
        Ok(AppState {
            db: pool,
            user: Arc::new(Mutex::new(AuthenticatedUser::default())),
        })
    }
}



// the application state
#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
    pub user: Arc<Mutex<AuthenticatedUser>>,
}


