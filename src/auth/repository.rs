use axum::extract::State;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::schema;
use crate::{auth::models::ListUser, common::Pool};

pub async fn full_auth(
    State(pool): State<Pool>,
    obj: String,
) -> Result<Option<ListUser>, Option<String>> {
    
    let mut conn = pool.get().await.unwrap();
    use schema::users::dsl::*;

    let user: Option<ListUser> = users
        .filter(email.eq(obj))
        .select(ListUser::as_select())
        .first::<ListUser>(&mut conn)
        .await
        .optional()
        .unwrap();

    Ok(user)
}
