use std::error::Error;

use url::Url;

pub fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find(':')
        .ok_or_else(|| format!("invalid KEY:value: no `:` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub fn parse_key_or_key_val<T, U>(
    s: &str,
) -> Result<(T, Option<U>), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    if s.contains(':') {
        let pos = s
            .find(':')
            .ok_or_else(|| format!("invalid KEY:value: no `:` found in `{s}`"))?;
        Ok((s[..pos].parse()?, Some(s[pos + 1..].parse()?)))
    } else {
        Ok((s.parse()?, None))
    }
}

pub fn parse_url(s: &str) -> Result<String, String> {
    let s = if !s.starts_with("http://") && !s.starts_with("https://") {
        format!("http://{}", s)
    } else {
        s.to_string()
    };
    let url = Url::parse(&s);

    match url {
        Ok(url) => {
            if !url.host().is_some() {
                return Err("Invalid URL".to_string());
            }
            Ok(url.to_string())
        }
        Err(_) => Err("Invalid URL".to_string()),
    }
}

pub fn parse_header(s: &str) -> Result<String, String> {
    // key: value
    let parts = s.split(":").collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("Invalid header".to_string());
    }
    Ok(s.to_string())
}

pub fn parse_cookie(s: &str) -> Result<String, String> {
    // key=value
    let parts = s.split("=").collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err("Invalid cookie".to_string());
    }
    Ok(s.to_string())
}

pub fn parse_method(s: &str) -> Result<String, String> {
    let methods = vec![
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "TRACE", "CONNECT",
    ];
    let s = s.to_uppercase();
    if methods.contains(&s.as_str()) {
        Ok(s.to_string())
    } else {
        Err("Invalid HTTP method".to_string())
    }
}
