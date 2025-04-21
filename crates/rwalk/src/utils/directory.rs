use reqwest::StatusCode;
use tracing::debug;

use crate::worker::utils::RwalkResponse;

pub fn check(response: &RwalkResponse) -> bool {
    let status = StatusCode::from_u16(response.status as u16).unwrap();
    if status.is_redirection() {
        // status code is 3xx
        match response.headers.get("Location") {
            // and has a Location header
            Some(loc) => {
                // get absolute redirect Url based on the already known base url
                debug!("Location header: {:?}", loc);

                if let Ok(abs_url) = response.url.join(&loc.as_immutable_string_ref().unwrap()) {
                    if format!("{}/", response.url) == abs_url.as_str() {
                        // if current response's Url + / == the absolute redirection
                        // location, we've found a directory suitable for recursion
                        debug!("found directory suitable for recursion: {}", response.url);
                        return true;
                    }
                }
            }
            None => {
                debug!(
                    "expected Location header, but none was found: {:?}",
                    response
                );
                return false;
            }
        }
    } else if status.is_success()
        || matches!(
            status,
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED // 403, 401 ; a little bit of a hack but it works most of the time
        )
    {
        // status code is 2xx or 403, need to check if it ends in /

        if response.url.as_str().ends_with('/') {
            debug!("{} is directory suitable for recursion", response.url);
            return true;
        }
    }

    false
}
