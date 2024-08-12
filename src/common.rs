use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{Html, Response};
use tera::Tera;

use crate::utils::message::Message;

pub type Templates = Arc<Tera>;

#[derive(Debug, Clone, Default)]
pub struct AuthenticatedUser {
    pub email: String,
    pub purpose: String,
}


// #[async_trait]
// impl FromRequestParts<AppState> for AuthenticatedUser
// {
//     type Rejection = Redirect;
//
//     async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
//         let headers = HeaderMap::from_request_parts(parts, state)
//             .await
//             .map_err(|_| Redirect::to("/account/login"))?;
//
//         let cookie = headers
//             .get(COOKIE)
//             .and_then(|value| value.to_str().ok())
//             .unwrap_or_default();
//
//         let token = match extract_cookie_value(cookie, "visit") {
//             Some(token) => token,
//             _ => return Err(Redirect::to("/account/login")),
//         };
//
//         let claims = match decode_token(token).await {
//             Ok(Some(claims)) => claims,
//             _ => return Err(Redirect::to("/account/login")),
//         };
//         info!("{:?}",claims);
//
//         let user = details(state.db.clone(), claims.email.clone()).await;
//
//         match user {
//             Ok(user) => {
//                 let user = AuthenticatedUser {
//                     email: user.email,
//                     purpose: "".to_string(),
//                 };
//                 let mut state_user = state.user.lock().expect("mutex was poisoned");
//                 *state_user = user.clone();
//                 Ok(user)
//             }
//             _ => Err(Redirect::to("/account/login"))
//         }
//     }
// }
// an extractor that wraps another and measures how long time it takes to run
#[derive(Debug)]
pub struct Timing<E> {
    pub extractor: E,
    pub duration: Duration,
}

// we must implement both `FromRequestParts`
#[async_trait]
impl<S, T> FromRequestParts<S> for Timing<T>
where
    S: Send + Sync,
    T: FromRequestParts<S>,
{
    type Rejection = T::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let start = Instant::now();
        let extractor = T::from_request_parts(parts, state).await?;
        let duration = start.elapsed();
        Ok(Timing {
            extractor,
            duration,
        })
    }
}

//
// #[async_trait]
// impl FromRequestParts<AppState> for AuthenticatedUser {
//     type Rejection = ();
//
//     async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
//         let headers = HeaderMap::from_request_parts(parts, state)
//             .await
//             .map_err(|_| ())?;
//
//         let cookie = headers
//             .get(COOKIE)
//             .and_then(|value| value.to_str().ok())
//             .unwrap_or_default();
//
//         let token = match extract_cookie_value(cookie, "visit") {
//             Some(token) => token,
//             _ => return Err(()),
//         };
//
//         let claims = match decode_token(token).await {
//             Ok(Some(claims)) => claims,
//             _ => return Err(()),
//         };
//         info!("{:?}", claims);
//
//         let user = details(state.db.clone(), claims.email.clone()).await;
//
//         match user {
//             Ok(user) => {
//                 let user = AuthenticatedUser {
//                     email: user.email,
//                     purpose: "".to_string(),
//                 };
//                 let mut state_user = state.user.lock().expect("mutex was poisoned");
//                 *state_user = user.clone();
//                 Ok(user)
//             }
//             _ => Err(()),
//         }
//     }
// }

pub async fn html_err(
    templates: &Arc<Tera>,
    name: &str,
    context: &mut tera::Context,
    message: String,
) -> Html<String> {
    // Insert the error message into the context
    context.insert("messages", &vec![Message {
        content: message.to_string(),
        tags: "danger".to_string(),
    }]);

    // Render the template with the updated context
    Html(templates.render(name, context).unwrap_or_else(|_| {
        // Handle potential template rendering errors
        "<h1>Error rendering template</h1>".to_string()
    }))
}

pub fn build_redirect_with_cookie(token: &str, max_age:String, loc: &str) -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", loc)
        .header(
            "Set-Cookie",
            format!(
                "visit={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
                token, max_age
            ),
        )
        .body(Body::from(""))
        .unwrap()
}
