use serde::Deserialize;

use core::marker::PhantomData;
use serde::de::{Deserializer, MapAccess, Visitor};

use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};

pub struct InputBody(pub Bytes);

#[async_trait]
impl<S> FromRequest<S> for InputBody
where
    Bytes: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let body = Bytes::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;
        Ok(Self(body))
    }
}

struct ListInput<V>(PhantomData<fn() -> V>);

impl<'de, V: Deserialize<'de>> Visitor<'de> for ListInput<V> {
    type Value = Option<Vec<V>>;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("err..")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut s: Option<Vec<V>> = Some(Vec::with_capacity(map.size_hint().unwrap_or(0)));

        while let Some((key, value)) = map.next_entry::<String, V>()? {
            if key == "list" {
                s.as_mut().expect("REASON").push(value);
            }
        }
        Ok(Some(s.expect("REASON")))
    }
}
pub fn deserialize_list<'de, D, V>(deserializer: D) -> Result<Option<Vec<V>>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    deserializer.deserialize_map(ListInput(PhantomData))
}
