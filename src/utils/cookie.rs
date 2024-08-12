use cookie::Cookie;

pub fn extract_cookie_value(cookie_str: &str, cookie_name: &str) -> Option<String> {
    cookie_str
        .split(';')
        .filter_map(|cookie| Cookie::parse(cookie.trim()).ok())
        .find(|cookie| cookie.name() == cookie_name)
        .map(|cookie| cookie.value().to_string())
}