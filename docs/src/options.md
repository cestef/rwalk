# Options

This document contains the help content for the `rwalk` command-line program.

**Command Overview:**

* [`rwalk`↴](#rwalk)

## `rwalk`

A blazing fast web directory scanner

**Usage:** `rwalk [OPTIONS] [URL] [FILE:KEY]...`

###### **Arguments:**

* `<URL>` — Target URL
* `<FILE:KEY>` — Wordlist(s)

###### **Options:**

* `-m`, `--mode <MODE>` — Crawl mode

  Possible values: `recursive`, `recursion`, `r`, `classic`, `c`, `spider`, `s`

* `--force` — Force scan even if the target is not responding

  Possible values: `true`, `false`

* `--hit-connection-errors` — Consider connection errors as a hit

  Possible values: `true`, `false`

* `-t`, `--threads <THREADS>` — Number of threads to use
* `-d`, `--depth <DEPTH>` — Crawl recursively until given depth
* `-o`, `--output <FILE>` — Output file
* `--pretty` — Pretty format the output (only JSON)

  Possible values: `true`, `false`

* `--timeout <TIMEOUT>` — Request timeout in seconds

  Default value: `10`
* `-u`, `--user-agent <USER_AGENT>` — User agent
* `-X`, `--method <METHOD>` — HTTP method

  Default value: `GET`
* `-D`, `--data <DATA>` — Data to send with the request
* `-H`, `--headers <key:value>` — Headers to send
* `-C`, `--cookies <key=value>` — Cookies to send
* `-R`, `--follow-redirects <COUNT>` — Follow redirects

  Default value: `5`
* `-c`, `--config <CONFIG>` — Configuration file
* `--throttle <THROTTLE>` — Request throttling (requests per second) per thread
* `-M`, `--max-time <MAX_TIME>` — Max time to run (will abort after given time) in seconds
* `--no-color` — Don't use colors You can also set the NO_COLOR environment variable

  Possible values: `true`, `false`

* `-q`, `--quiet` — Quiet mode

  Possible values: `true`, `false`

* `-i`, `--interactive` — Interactive mode

  Possible values: `true`, `false`

* `--insecure` — Insecure mode, disables SSL certificate validation

  Possible values: `true`, `false`

* `--show <SHOW>` — Show response additional body information
* `-r`, `--resume` — Resume from a saved file

  Possible values: `true`, `false`

* `--save-file <FILE>` — Custom save file

  Default value: `.rwalk.json`
* `--no-save` — Don't save the state in case you abort

  Possible values: `true`, `false`

* `--keep-save` — Keep the save file after finishing when using --resume

  Possible values: `true`, `false`

* `-T`, `--transform <TRANSFORM>` — Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"
* `-w`, `--wordlist-filter <KEY:FILTER>` — Wordlist filtering: "contains", "starts", "ends", "regex", "length"
* `-f`, `--filter <KEY:FILTER>` — Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"
* `--or` — Treat filters as or instead of and

  Possible values: `true`, `false`

* `--force-recursion` — Force the recursion over non-directories

  Possible values: `true`, `false`

* `--directory-script <DIRECTORY_SCRIPT>` — Override the default directory detection method with your own rhai script
* `--request-file <FILE>` — Request file (.http, .rest)
* `-P`, `--proxy <URL>` — Proxy URL
* `--proxy-auth <USER:PASS>` — Proxy username and password
* `--subdomains` — Allow subdomains to be scanned in spider mode

  Possible values: `true`, `false`

* `--external` — Allow external domains to be scanned in spider mode (Warning: this can generate a lot of traffic)

  Possible values: `true`, `false`

* `-a`, `--attributes <ATTRIBUTES>` — Attributes to be crawled in spider mode
* `--scripts <SCRIPTS>` — Scripts to run after each request
* `--ignore-scripts-errors` — Ignore scripts errors

  Possible values: `true`, `false`

* `--generate-markdown` — Generate markdown help - for developers

  Possible values: `true`, `false`

* `--completions <COMPLETIONS>`
* `--open-config` — Open the config in the default editor (EDITOR)

  Possible values: `true`, `false`

* `--default-config` — Print the default config in TOML format. Useful for debugging and creating your own config

  Possible values: `true`, `false`




<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

