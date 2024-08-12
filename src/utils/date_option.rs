use once_cell::sync::Lazy;

static MAX_AGE_COOKIE: Lazy<String> = Lazy::new(|| {
    dotenv::var("MAX_AGE_COOKIE").expect("DATABASE_URL must be set")
});

pub mod option_date {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(date: &Option<NaiveDate>, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        if let Some(ref d) = *date {
            return s.serialize_str(&d.format("%Y-%m-%d").to_string());
        }
        s.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where D: Deserializer<'de>
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(
                NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)?,
            ));
        }
        Ok(None)
    }
}

pub fn get_max_age_seconds() -> String {
    let max_age: i64 = MAX_AGE_COOKIE
        .parse()
        .expect("MAX_AGE_COOKIE must be an integer");
    let max_age_seconds = 3600 * max_age;
    max_age_seconds.to_string()
}