use std::collections::BTreeMap;

use colored::Colorize;
use reqwest::StatusCode;
use rhai::plugin::*;

use crate::{
    cli::opts::Opts,
    utils::constants::{ERROR, WARNING},
};
use color_eyre::eyre::Result;

use super::ScriptingResponse;

pub fn print_error(
    opts: &Opts,
    print_fn: impl FnOnce(String) -> Result<()>,
    url: &str,
    err: reqwest::Error,
) -> Result<()> {
    if !opts.quiet {
        if err.is_timeout() {
            print_fn(format!(
                "{} {} {}",
                ERROR.to_string().red(),
                "Timeout reached".bold(),
                url
            ))?;
        } else if err.is_redirect() {
            print_fn(format!(
                "{} {} {} {}",
                WARNING.to_string().yellow(),
                "Redirect limit reached".bold(),
                url,
                "Check --follow-redirects".dimmed()
            ))?;
        } else if err.is_connect() {
            print_fn(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Connection error".bold(),
                url,
                format!("({})", err).dimmed()
            ))?;
        } else if err.is_request() {
            print_fn(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Request error".bold(),
                url,
                format!("({})", err).dimmed()
            ))?;
        } else {
            print_fn(format!(
                "{} {} {} {}",
                ERROR.to_string().red(),
                "Unknown Error".bold(),
                url,
                format!("({})", err).dimmed()
            ))?;
        }
    }
    Ok(())
}

pub fn is_html_directory(body: &str) -> bool {
    let body = body.to_lowercase();
    // Apache
    if body.contains("index of") {
        return true;
    }
    // Nginx
    if body.contains("name=\"description\" content=\"nginx directory listing\"") {
        return true;
    }
    // ASP.NET
    if body.contains("directory listing -- /") {
        return true;
    }
    // Tomcat
    if body.contains("directory listing for /") {
        return true;
    }

    false
}

pub fn is_directory(
    opts: &Opts,
    response: &reqwest::Response,
    body: String,
    progress: &indicatif::ProgressBar,
) -> bool {
    if let Some(directory_script) = opts.directory_script.as_ref() {
        let mut engine = rhai::Engine::new();
        let mut scope = rhai::Scope::new();
        let headers = response
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.as_str().to_string(),
                    value.to_str().unwrap().to_string(),
                )
            })
            .collect::<BTreeMap<String, String>>();
        scope.push(
            "response",
            ScriptingResponse {
                status_code: response.status().as_u16(),
                headers: headers.clone().into(),
                body: body.clone(),
                url: response.url().as_str().to_string(),
            },
        );
        scope.push("opts", opts.clone());
        engine.build_type::<ScriptingResponse>();
        let engine_opts = opts.clone();
        let engine_progress = progress.clone();
        engine.on_print(move |s| {
            if !engine_opts.quiet {
                engine_progress.println(s);
            }
        });

        let res = engine
            .eval_file_with_scope::<Dynamic>(&mut scope, directory_script.into())
            .map_err(|e| {
                progress.println(format!(
                    "{} {} {}",
                    ERROR.to_string().red(),
                    "Error running script".bold(),
                    e
                ));
                e
            });
        if let Ok(res) = res {
            if let Ok(res) = res.as_bool() {
                return res;
            } else {
                progress.println(format!(
                    "{} {}",
                    ERROR.to_string().red(),
                    "Script did not return a boolean".bold()
                ));
            }
        }
    }
    if let Some(content_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        if content_type.to_str().unwrap().starts_with("text/html") {
            // log::debug!("{} is HTML", response.url());
            if is_html_directory(&body) {
                log::debug!("{} is directory suitable for recursion", response.url());
                return true;
            }
        }
    }
    if response.status().is_redirection() {
        // status code is 3xx
        match response.headers().get("Location") {
            // and has a Location header
            Some(loc) => {
                // get absolute redirect Url based on the already known base url
                log::debug!("Location header: {:?}", loc);

                if let Ok(loc_str) = loc.to_str() {
                    if let Ok(abs_url) = response.url().join(loc_str) {
                        if format!("{}/", response.url()) == abs_url.as_str() {
                            // if current response's Url + / == the absolute redirection
                            // location, we've found a directory suitable for recursion
                            log::debug!(
                                "found directory suitable for recursion: {}",
                                response.url()
                            );
                            return true;
                        }
                    }
                }
            }
            None => {
                log::debug!(
                    "expected Location header, but none was found: {:?}",
                    response
                );
                return false;
            }
        }
    } else if response.status().is_success()
        || matches!(
            response.status(),
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED // 403, 401 ; a little bit of a hack but it works most of the time
        )
    {
        // status code is 2xx or 403, need to check if it ends in /

        if response.url().as_str().ends_with('/') {
            log::debug!("{} is directory suitable for recursion", response.url());
            return true;
        }
    }

    false
}
