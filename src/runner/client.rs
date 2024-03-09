use std::path::Path;

use anyhow::{Context, Result};
use http_rest_file::{model::Header, Parser};
use reqwest::{
    header::{HeaderMap, HeaderName},
    redirect::Policy,
    Proxy,
};

use crate::{
    cli::opts::Opts,
    utils::constants::{DEFAULT_FOLLOW_REDIRECTS, DEFAULT_METHOD, DEFAULT_TIMEOUT},
};

pub fn build(opts: &Opts) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    opts.headers.clone().iter().for_each(|header| {
        let mut header = header.splitn(2, ':');
        let key = header.next().unwrap().trim();
        let value = header.next().unwrap().trim();
        headers.insert(key.parse::<HeaderName>().unwrap(), value.parse().unwrap());
    });
    opts.cookies.clone().iter().for_each(|cookie| {
        let mut cookie = cookie.splitn(2, '=');
        let key = cookie.next().unwrap().trim();
        let value = cookie.next().unwrap().trim();
        headers.extend(vec![(
            reqwest::header::COOKIE,
            format!("{}={}", key, value).parse().unwrap(),
        )]);
    });
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(opts.insecure)
        .user_agent(
            opts.user_agent
                .clone()
                .unwrap_or(format!("rwalk/{}", env!("CARGO_PKG_VERSION"))),
        )
        .default_headers(headers)
        .redirect(
            if opts.follow_redirects.unwrap_or(DEFAULT_FOLLOW_REDIRECTS) > 0 {
                Policy::limited(opts.follow_redirects.unwrap_or(DEFAULT_FOLLOW_REDIRECTS))
            } else {
                Policy::none()
            },
        )
        .timeout(std::time::Duration::from_secs(
            opts.timeout.unwrap_or(DEFAULT_TIMEOUT) as u64,
        ));
    let client = if let Some(proxy) = opts.proxy.clone() {
        let proxy = Proxy::all(proxy)?;
        if let Some(auth) = opts.proxy_auth.clone() {
            let mut auth = auth.splitn(2, ':');
            let username = auth.next().unwrap().trim();
            let password = auth.next().unwrap().trim();

            let proxy = proxy.basic_auth(username, password);
            client.proxy(proxy)
        } else {
            client.proxy(proxy)
        }
    } else {
        client
    };

    Ok(client.build()?)
}

pub fn get_sender(
    method: Option<String>,
    body: Option<String>,
    url: &str,
    client: &reqwest::Client,
) -> reqwest::RequestBuilder {
    match method.unwrap_or(DEFAULT_METHOD.to_string()).as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url).body(body.unwrap_or("".to_string())),
        "PUT" => client.put(url).body(body.unwrap_or("".to_string())),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, url),
        "TRACE" => client.request(reqwest::Method::TRACE, url),
        "CONNECT" => client.request(reqwest::Method::CONNECT, url),
        _ => panic!("Invalid HTTP method"),
    }
}

pub fn build_request(opts: &Opts, url: &str, client: &reqwest::Client) -> Result<reqwest::Request> {
    if let Some(request_file) = &opts.request_file {
        let path = Path::new(request_file);
        let model = Parser::parse_file(path).context("Failed to parse request file")?;
        let request = model.requests.first().context("No request found in file")?;
        let sender = get_sender(
            Some(
                request
                    .request_line
                    .method
                    .get_cloned_or_computed()
                    .to_string(),
            ),
            if request.body.is_present() {
                Some(request.body.to_string())
            } else {
                None
            },
            url,
            client,
        );
        let mut headers = HeaderMap::new();
        request.headers.iter().for_each(|Header { key, value }| {
            headers.insert(key.parse::<HeaderName>().unwrap(), value.parse().unwrap());
        });
        Ok(sender.headers(headers).build()?)
    } else {
        let sender = get_sender(opts.method.clone(), opts.data.clone(), url, client);
        Ok(sender.build()?)
    }
}
