use reqwest::StatusCode;
use tracing::debug;

use crate::worker::utils::RwalkResponse;

const HTML_DIRECTORY_INDICATORS: [&str; 4] = [
    "index of",
    "nginx directory listing",
    "directory listing -- /",
    "directory listing for /",
];

pub fn is_html_directory(body: &str) -> bool {
    return HTML_DIRECTORY_INDICATORS
        .iter()
        .any(|&indicator| body.contains(&indicator.to_lowercase()));
}

pub fn check(response: &RwalkResponse) -> bool {
    // if let Some(content_type) = response.headers.get("Content-Type") {
    //     if content_type.starts_with("text/html") {
    //         if is_html_directory(&response.body.as_ref().unwrap()) {
    //             debug!("{} is directory suitable for recursion", response.url());
    //             return true;
    //         }
    //     }
    // }
    if response.status.is_redirection() {
        // status code is 3xx
        match response.headers.get("Location") {
            // and has a Location header
            Some(loc) => {
                // get absolute redirect Url based on the already known base url
                debug!("Location header: {:?}", loc);

                if let Ok(abs_url) = response.url.join(&loc) {
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
    } else if response.status.is_success()
        || matches!(
            response.status,
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
