use dashmap::DashMap;

use crate::worker::utils::RwalkResponse;

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
