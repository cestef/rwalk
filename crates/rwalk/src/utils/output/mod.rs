use dashmap::DashMap;
use url::Url;

use crate::worker::utils::{ResponseType, RwalkResponse};

pub fn csv(data: &DashMap<String, RwalkResponse>) -> String {
    let mut csv = String::new();
    csv.push_str("url,status,depth,type,time,body\n");

    for e in data.iter() {
        let (url, response) = e.pair();
        csv.push_str(&format!(
            "{},{},{:?},{:?},{},{}\n",
            url, response.status, response.depth, response.r#type, response.time, response.body,
        ));
    }

    csv
}

pub fn txt(data: &DashMap<String, RwalkResponse>) -> String {
    let mut txt = String::new();

    for e in data.iter() {
        let (url, response) = e.pair();
        txt.push_str(&format!(
            "{}\n\tstatus: {}\n\tdepth: {:?}\n\ttype: {:?}\n\ttime: {}ms\n\tbody: {}\n",
            url, response.status, response.depth, response.r#type, response.time, response.body,
        ));
    }

    txt
}

pub fn md(data: &DashMap<String, RwalkResponse>) -> String {
    let mut md = String::new();
    md.push_str("| url | status | depth | type | time | body |\n");
    md.push_str("| --- | ------ | ----- | ---- | ---- | ---- |\n");

    for e in data.iter() {
        let (url, response) = e.pair();
        md.push_str(&format!(
            "| {} | {} | {:?} | {:?} | {}ms | {} |\n",
            url, response.status, response.depth, response.r#type, response.time, response.body,
        ));
    }

    md
}

pub fn html(results: &DashMap<String, RwalkResponse>, base_url: &Url) -> crate::Result<String> {
    let mut endpoints = Vec::new();
    let mut status_counts = std::collections::HashMap::new();
    let mut content_types = std::collections::HashMap::new();
    let mut directories = 0;
    let mut files = 0;
    let mut errors = 0;

    for entry in results.iter() {
        let (url, response) = (entry.key(), entry.value());

        let status = response.status as u16;
        *status_counts.entry(status).or_insert(0) += 1;

        match &response.r#type {
            ResponseType::Directory => {
                directories += 1;
            }
            ResponseType::File(content_type) => {
                files += 1;
                if let Some(ct) = content_type {
                    *content_types.entry(ct.clone()).or_insert(0) += 1;
                }
            }
            ResponseType::Error => {
                errors += 1;
            }
        }

        endpoints.push((
            url.clone(),
            status,
            response.time,
            response.r#type.clone(),
            response.body.len(),
        ));
    }

    endpoints.sort_by(|(a, _, _, _, _), (b, _, _, _, _)| a.cmp(b));

    let mut table_rows = String::new();
    for (url, status, time, response_type, size) in endpoints {
        let type_str = match &response_type {
            ResponseType::Directory => "Directory".to_string(),
            ResponseType::File(Some(ct)) => format!("File ({})", ct),
            ResponseType::File(None) => "File".to_string(),
            ResponseType::Error => "Error".to_string(),
        };

        let status_class = match status {
            200..=299 => "success",
            300..=399 => "redirect",
            400..=499 => "client-error",
            500..=599 => "server-error",
            _ => "unknown",
        };

        table_rows.push_str(&format!(
            r#"<tr>
                <td><a href="{url}" target="_blank">{url}</a></td>
                <td class="{status_class}">{status}</td>
                <td>{time_ms}ms</td>
                <td>{type_str}</td>
                <td>{size}</td>
            </tr>"#,
            url = url,
            status_class = status_class,
            status = status,
            time_ms = time / 1000,
            type_str = type_str,
            size = size,
        ));
    }

    let status_chart_data = status_counts
        .iter()
        .map(|(&status, &count)| format!("{{status: {}, count: {}}}", status, count))
        .collect::<Vec<_>>()
        .join(",");

    let content_chart_data = content_types
        .iter()
        .map(|(content_type, &count)| {
            let short_type = content_type.split('/').last().unwrap_or(content_type);
            format!("{{type: '{}', count: {}}}", short_type, count)
        })
        .collect::<Vec<_>>()
        .join(",");

    let html = format!(
        include_str!("report_template.html"),
        base_url = base_url,
        date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        total = results.len(),
        directories = directories,
        files = files,
        errors = errors,
        status_summary = status_counts
            .iter()
            .map(|(&status, &count)| format!("<p>{}: <strong>{}</strong></p>", status, count))
            .collect::<String>(),
        table_rows = table_rows,
        status_chart_data = status_chart_data,
        content_chart_data = content_chart_data,
    );

    Ok(html)
}
