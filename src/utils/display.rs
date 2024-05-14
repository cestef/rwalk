use super::color_for_status_code;
use colored::Colorize;

pub fn display_range_status(mut status: String) -> String {
    if status.contains('-') {
        status = status
            .split('-')
            .map(|x| match x.parse::<u16>() {
                Ok(x) => color_for_status_code(x.to_string(), x),
                Err(_) => x.to_string(),
            })
            .collect::<Vec<_>>()
            .join("-")
            .to_string();
    } else if let Some(stripped) = status.strip_prefix('>') {
        status = format!(
            ">{}",
            color_for_status_code(stripped.to_string(), stripped.parse().unwrap_or_default())
        );
    } else if let Some(stripped) = status.strip_prefix('<') {
        status = format!(
            "<{}",
            color_for_status_code(stripped.to_string(), stripped.parse().unwrap_or_default())
        );
    } else {
        status = color_for_status_code(status.to_string(), status.parse().unwrap_or_default());
    }

    status
}

pub fn display_range(range: String) -> String {
    range
        .split(',')
        .map(|x| {
            if let Some(stripped) = x.strip_prefix('>') {
                format!(">{}", stripped.blue())
            } else if let Some(stripped) = x.strip_prefix('<') {
                format!("<{}", stripped.blue())
            } else {
                let parts = x.split('-').collect::<Vec<_>>();
                if parts.len() == 2 {
                    let start = parts[0].parse::<u16>().unwrap_or_default();
                    let end = parts[1].parse::<u16>().unwrap_or_default();
                    format!("{}-{}", start.to_string().blue(), end.to_string().blue())
                } else if let Ok(x) = x.parse::<u16>() {
                    x.to_string().blue().to_string()
                } else {
                    x.to_string()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn color_n(s: String, n: usize) -> String {
    match n % 5 {
        0 => s.bold().green().to_string(),
        1 => s.bold().yellow().to_string(),
        2 => s.bold().red().to_string(),
        3 => s.bold().cyan().to_string(),
        _ => s.bold().magenta().to_string(),
    }
}
