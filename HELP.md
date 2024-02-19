# Command-Line Help for `rwalk`

This document contains the help content for the `rwalk` command-line program.

**Command Overview:**

* [`rwalk`↴](#rwalk)

## `rwalk`

A blazing fast web directory scanner

**Usage:** `rwalk [OPTIONS] [URL] [WORDLISTS]...`

###### **Arguments:**

* `<URL>` — Target URL
* `<WORDLISTS>` — Wordlist(s)

###### **Options:**

* `-m`, `--mode <MODE>` — Crawl mode

  Default value: `recursive`

  Possible values: `recursive`, `recursion`, `r`, `classic`, `c`

* `-p`, `--permutations` — Permutations mode

  Possible values: `true`, `false`

* `-t`, `--threads <THREADS>` — Number of threads to use
* `-d`, `--depth <DEPTH>` — Crawl recursively until given depth

  Default value: `1`
* `-o`, `--output <FILE>` — Output file
* `--timeout <TIMEOUT>` — Request timeout in seconds

  Default value: `10`
* `-u`, `--user-agent <USER_AGENT>` — User agent
* `-X`, `--method <METHOD>` — HTTP method

  Default value: `GET`
* `-D`, `--data <DATA>` — Data to send with the request
* `-H`, `--headers <key:value>` — Headers to send
* `-c`, `--cookies <key=value>` — Cookies to send
* `--fuzz-key <FUZZ_KEY>` — Change the default fuzz-key

  Default value: `$`
* `-R`, `--follow-redirects <COUNT>` — Follow redirects

  Default value: `2`
* `--throttle <THROTTLE>` — Request throttling (requests per second) per thread

  Default value: `0`
* `-M`, `--max-time <MAX_TIME>` — Max time to run (will abort after given time) in seconds
* `--no-color` — Don't use colors You can also set the NO_COLOR environment variable

  Possible values: `true`, `false`

* `-q`, `--quiet` — Quiet mode

  Possible values: `true`, `false`

* `-i`, `--interactive` — Interactive mode

  Possible values: `true`, `false`

* `--insecure` — Insecure mode, disables SSL certificate validation

  Possible values: `true`, `false`

* `--show <SHOW>` — Show response additional body information: "length", "hash", "headers_length", "headers_hash", "body", "headers"
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
* `-f`, `--filter <KEY:FILTER>` — Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash"
* `--or` — Treat filters as or instead of and

  Possible values: `true`, `false`

* `-P`, `--proxy <URL>` — Proxy URL
* `--proxy-auth <USER:PASS>` — Proxy username and password
* `--generate-markdown` — Generate markdown help - for developers

  Possible values: `true`, `false`




<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

