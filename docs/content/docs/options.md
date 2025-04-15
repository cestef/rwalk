+++
title = "Options"
weight = 20
+++

**Usage:** `rwalk [OPTIONS] [URL] [WORDLISTS]...`

###### **Arguments:**

* `<URL>` — URL to scan
* `<WORDLISTS>` — Wordlist file(s) to use, `path[:key]`

###### **Options:**

* `-T`, `--threads <THREADS>` — Number of threads to use, defaults to `num_cores * 5`

  Default value: `55`
* `--throttle <THROTTLE>` — Request rate limit in requests per second
* `-m`, `--mode <MODE>` — Fuzzing mode

  Default value: `recursive`

  Possible values:
  - `recursive`:
    Recursively fuzz the target, increasing the depth with each request
  - `template`:
    Use a template to generate payloads, replacing placeholders with wordlist values

* `--http1` — Only use HTTP/1
* `--http2` — Only use HTTP/2
* `-d`, `--depth <DEPTH>` — Maximum depth in recursive mode

  Default value: `0`
* `-r`, `--retries <RETRIES>` — Maximum retries for failed requests

  Default value: `3`
* `--retry-codes <RETRY_CODES>` — What status codes to retry on
* `--force` — Force the scan, even if the target is unreachable
* `--force-recursion` — Force the recursion, even if the URL is not detected as a directory
* `-X`, `--method <METHOD>` — HTTP method to use

  Default value: `GET`

  Possible values: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`, `TRACE`

* `-H`, `--headers <HEADER>` — Headers to send with the request, `name:value`
* `-s`, `--show <SHOW>` — Extra information to display on hits
* `-o`, `--output <OUTPUT>` — Save responses to a file, supported: `json`, `csv`, `txt`, `md`
* `--bell` — Ring the terminal bell on hits
* `-f`, `--filters <EXPR>` — List of filters to apply to responses, see `--list-filters`
* `-t`, `--transforms <TRANSFORM>` — List of transformations to apply to wordlists, see `--list-transforms`
* `-w`, `--wordlist-filter <EXPR>` — Wordlist filters, see `--list-filters`
* `--resume` — Resume from previous session
* `--no-save` — Don't save state on `Ctrl+C`
* `-c`, `--config <CONFIG>` — Load configuration from a file, merges with command line arguments
* `--list <LIST>` — List both available filters and wordlist transforms

  Possible values: `filters`, `transforms`, `all`




