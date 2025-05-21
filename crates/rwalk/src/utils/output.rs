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
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>rwalk Scan Report - {base_url}</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 1200px;
            margin: 0 auto;
            padding: 1rem;
        }}
        header {{
            margin-bottom: 2rem;
            border-bottom: 1px solid #eee;
            padding-bottom: 1rem;
        }}
        .summary {{
            display: flex;
            flex-wrap: wrap;
            gap: 2rem;
            margin-bottom: 2rem;
        }}
        .summary-box {{
            flex: 1;
            min-width: 250px;
            padding: 1rem;
            border-radius: 8px;
            box-shadow: 0 2px 5px rgba(0,0,0,0.1);
        }}
        .charts {{
            display: flex;
            flex-wrap: wrap;
            gap: 2rem;
            margin-bottom: 2rem;
        }}
        .chart {{
            flex: 1;
            min-width: 300px;
            height: 300px;
            padding: 1rem;
            border-radius: 8px;
            box-shadow: 0 2px 5px rgba(0,0,0,0.1);
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 2rem;
        }}
        th, td {{
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background-color: #f8f8f8;
            font-weight: 600;
        }}
        tr:hover {{
            background-color: #f5f5f5;
        }}
        .success {{ color: #28a745; }}
        .redirect {{ color: #007bff; }}
        .client-error {{ color: #ffc107; }}
        .server-error {{ color: #dc3545; }}
        .unknown {{ color: #6c757d; }}
        .filter-controls {{
            margin: 1rem 0;
            display: flex;
            gap: 1rem;
            align-items: center;
        }}
        .search {{
            padding: 0.5rem;
            flex: 1;
            max-width: 500px;
        }}
        footer {{
            margin-top: 3rem;
            text-align: center;
            font-size: 0.9rem;
            color: #666;
        }}
    </style>
</head>
<body>
    <header>
        <h1>rwalk Scan Report</h1>
        <p>Base URL: <a href="{base_url}" target="_blank">{base_url}</a></p>
        <p>Scan Date: {date}</p>
    </header>
    
    <div class="summary">
        <div class="summary-box">
            <h2>Results Summary</h2>
            <p>Total Endpoints: <strong>{total}</strong></p>
            <p>Directories: <strong>{directories}</strong></p>
            <p>Files: <strong>{files}</strong></p>
            <p>Errors: <strong>{errors}</strong></p>
        </div>
        
        <div class="summary-box">
            <h2>Status Code Distribution</h2>
            <div id="status-summary">
                {status_summary}
            </div>
        </div>
    </div>
    
    <div class="charts">
        <div class="chart">
            <canvas id="statusChart"></canvas>
        </div>
        <div class="chart">
            <canvas id="contentTypeChart"></canvas>
        </div>
    </div>
    
    <h2>Discovered Endpoints</h2>
    
    <div class="filter-controls">
        <input type="text" id="searchInput" class="search" placeholder="Search endpoints...">
        <select id="statusFilter">
            <option value="all">All Status Codes</option>
            <option value="2xx">2xx Success</option>
            <option value="3xx">3xx Redirect</option>
            <option value="4xx">4xx Client Error</option>
            <option value="5xx">5xx Server Error</option>
        </select>
        <select id="typeFilter">
            <option value="all">All Types</option>
            <option value="directory">Directories</option>
            <option value="file">Files</option>
            <option value="error">Errors</option>
        </select>
    </div>
    
    <table id="endpointsTable">
        <thead>
            <tr>
                <th>URL</th>
                <th>Status</th>
                <th>Response Time</th>
                <th>Type</th>
                <th>Size (bytes)</th>
            </tr>
        </thead>
        <tbody>
            {table_rows}
        </tbody>
    </table>
    
    <footer>
        <p>Generated by rwalk - A blazingly fast web directory scanner</p>
    </footer>
    
    <script>
        // Status chart
        const statusCtx = document.getElementById('statusChart').getContext('2d');
        const statusData = [{status_chart_data}];
        
        new Chart(statusCtx, {{
            type: 'pie',
            data: {{
                labels: statusData.map(d => `${{d.status}} (${{d.count}})`),
                datasets: [{{
                    data: statusData.map(d => d.count),
                    backgroundColor: statusData.map(d => {{
                        if (d.status >= 200 && d.status < 300) return '#28a745';
                        if (d.status >= 300 && d.status < 400) return '#007bff';
                        if (d.status >= 400 && d.status < 500) return '#ffc107';
                        if (d.status >= 500 && d.status < 600) return '#dc3545';
                        return '#6c757d';
                    }}),
                }}]
            }},
            options: {{
                plugins: {{
                    title: {{
                        display: true,
                        text: 'Status Codes'
                    }}
                }}
            }}
        }});
        
        // Content type chart
        const contentCtx = document.getElementById('contentTypeChart').getContext('2d');
        const contentData = [{content_chart_data}];
        
        new Chart(contentCtx, {{
            type: 'bar',
            data: {{
                labels: contentData.map(d => d.type),
                datasets: [{{
                    label: 'File Types',
                    data: contentData.map(d => d.count),
                    backgroundColor: '#007bff',
                }}]
            }},
            options: {{
                plugins: {{
                    title: {{
                        display: true,
                        text: 'Content Types'
                    }}
                }},
                scales: {{
                    y: {{
                        beginAtZero: true,
                        ticks: {{
                            precision: 0
                        }}
                    }}
                }}
            }}
        }});
        
        // Table filtering
        const searchInput = document.getElementById('searchInput');
        const statusFilter = document.getElementById('statusFilter');
        const typeFilter = document.getElementById('typeFilter');
        const table = document.getElementById('endpointsTable');
        const rows = table.getElementsByTagName('tr');
        
        function filterTable() {{
            const searchTerm = searchInput.value.toLowerCase();
            const statusValue = statusFilter.value;
            const typeValue = typeFilter.value;
            
            for (let i = 1; i < rows.length; i++) {{
                const row = rows[i];
                const url = row.cells[0].textContent.toLowerCase();
                const statusCode = parseInt(row.cells[1].textContent);
                const typeText = row.cells[3].textContent.toLowerCase();
                
                let showRow = url.includes(searchTerm);
                
                // Status filtering
                if (statusValue !== 'all') {{
                    if (statusValue === '2xx' && (statusCode < 200 || statusCode >= 300)) showRow = false;
                    if (statusValue === '3xx' && (statusCode < 300 || statusCode >= 400)) showRow = false;
                    if (statusValue === '4xx' && (statusCode < 400 || statusCode >= 500)) showRow = false;
                    if (statusValue === '5xx' && (statusCode < 500 || statusCode >= 600)) showRow = false;
                }}
                
                // Type filtering
                if (typeValue !== 'all') {{
                    if (typeValue === 'directory' && !typeText.includes('directory')) showRow = false;
                    if (typeValue === 'file' && !typeText.includes('file')) showRow = false;
                    if (typeValue === 'error' && !typeText.includes('error')) showRow = false;
                }}
                
                row.style.display = showRow ? '' : 'none';
            }}
        }}
        
        searchInput.addEventListener('input', filterTable);
        statusFilter.addEventListener('change', filterTable);
        typeFilter.addEventListener('change', filterTable);
    </script>
</body>
</html>"#,
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
