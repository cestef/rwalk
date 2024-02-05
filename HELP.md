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

* `-t`, `--threads <THREADS>` — Number of threads to use
* `-d`, `--depth <DEPTH>` — Maximum depth to crawl

  Default value: `1`
* `-o`, `--output <FILE>` — Output file
* `-T`, `--timeout <TIMEOUT>` — Request timeout in seconds

  Default value: `10`
* `-u`, `--user-agent <USER_AGENT>` — User agent
* `-m`, `--method <METHOD>` — HTTP method

  Default value: `GET`
* `-d`, `--data <DATA>` — Data to send with the request
* `-H`, `--headers <key:value>` — Headers to send
* `-c`, `--cookies <key=value>` — Cookies to send
* `-R`, `--follow-redirects <COUNT>` — Follow redirects

  Default value: `2`
* `--throttle <THROTTLE>` — Request throttling (requests per second) per thread

  Default value: `0`
* `-M`, `--max-time <MAX_TIME>` — Max time to run (will abort after given time) in seconds
* `--no-color` — Don't use colors You can also set the NO_COLOR environment variable
* `-q`, `--quiet` — Quiet mode
* `-i`, `--interactive` — Interactive mode
* `-r`, `--resume` — Resume from a saved file
* `-f`, `--save-file <FILE>` — Custom save file

  Default value: `.rwalk.json`
* `--no-save` — Don't save the state in case you abort
* `-L`, `--transform-lower` — Wordlist to uppercase
* `-U`, `--transform-upper` — Wordlist to lowercase
* `-P`, `--transform-prefix <PREFIX>` — Append a prefix to each word
* `-S`, `--transform-suffix <SUFFIX>` — Append a suffix to each word
* `-C`, `--transform-capitalize` — Capitalize each word
* `--wordlist-filter-contains <STRING>` — Contains the specified string
* `--wordlist-filter-starts-with <STRING>` — Starts with the specified string
* `--wordlist-filter-ends-with <STRING>` — Ends with the specified string
* `--wordlist-filter-regex <REGEX>` — Matches the specified regex
* `--wordlist-filter-length <RANGE>` — Length range e.g.: 5, 5-10, 5,10,15, >5, <5
* `--filter-status-code <RANGE>` — Reponse status code, e.g.: 200, 200-300, 200,300,400, >200, <200

  Default value: `200-299,300-399,400-403,500-599`
* `--filter-contains <STRING>` — Contains the specified string
* `--filter-starts-with <STRING>` — Starts with the specified string
* `--filter-ends-with <STRING>` — Ends with the specified string
* `--filter-regex <REGEX>` — Matches the specified regex
* `--filter-length <RANGE>` — Response length e.g.: 100, >100, <100, 100-200, 100,200,300
* `--filter-time <RANGE>` — Response time range in milliseconds e.g.: >1000, <1000, 1000-2000
* `--proxy <URL>` — Proxy URL
* `--proxy-auth <USER:PASS>` — Proxy username and password


