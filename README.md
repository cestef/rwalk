<p align="center">
    <img src="assets/header.png" alt="rwalk" />
</p>

A blazing fast web directory scanner written in Rust. It's like [dirsearch](https://github.com/maurosoria/dirsearch) but faster and with less features.

## Features

- [x] Multi-threaded
- [x] Recursive directory scanning
- [x] Save progress to resume later
- [x] Cherry-pick responses (filter by status code, length, etc.)
- [x] Custom wordlists (merge multiple wordlists, filter out words, etc.) 
- [x] Write results to file (JSON, CSV, etc.)
- [x] Configurable request parameters (headers, cookies, etc.)
- [x] Request throttling
- [x] Proxy support

## From [crates.io](https://crates.io/crates/rwalk)

### Installation

```bash
cargo install rwalk
```

### Running

```bash
rwalk https://example.com path/to/wordlist.txt
```
## From source

### Installation

```bash
git clone https://github.com/cestef/rwalk.git
cd rwalk
```

### Running

**With [just](https://github.com/casey/just)**

```bash
just run https://example.com path/to/wordlist.txt
```

**With [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)**

```bash
cargo run --release -- https://example.com path/to/wordlist.txt
```

## Usage

You can run `rwalk --help` to see the usage information:

```text
Usage: rwalk [OPTIONS] <URL> <WORDLISTS>...

Arguments:
  <URL>           Target URL
  <WORDLISTS>...  Wordlist(s)

Options:
  -t, --threads <THREADS>         Number of threads to use
  -d, --depth <DEPTH>             Maximum depth to crawl [default: 1]
  -o, --output <FILE>             Output file
  -T, --timeout <TIMEOUT>         Request timeout in seconds [default: 10]
  -u, --user-agent <USER_AGENT>   User agent
  -q, --quiet                     Quiet mode
  -m, --method <METHOD>           HTTP method [default: GET]
  -d, --data <DATA>               Data to send with the request
  -H, --headers <key:value>       Headers to send
  -c, --cookies <key=value>       Cookies to send
  -R, --follow-redirects <COUNT>  Follow redirects [default: 0]
      --throttle <THROTTLE>       Request throttling (requests per second) per thread [default: 0]
      --no-color                  Don't use colors You can also set the NO_COLOR environment variable
  -h, --help                      Print help
  -V, --version                   Print version

Resume:
      --resume            Resume from a saved file
  -f, --save-file <FILE>  Custom save file [default: .rwalk.json]
      --no-save           Don't save the state in case you abort

Transformations:
  -L, --transform-lower            Wordlist to uppercase
  -U, --transform-upper            Wordlist to lowercase
  -P, --transform-prefix <PREFIX>  Append a prefix to each word
  -S, --transform-suffix <SUFFIX>  Append a suffix to each word
  -C, --transform-capitalize       Capitalize each word

Wordlist Filtering:
      --wordlist-filter-contains <STRING>
          Contains the specified string [aliases: wfc]
      --wordlist-filter-starts-with <STRING>
          Start with the specified string [aliases: wfs]
      --wordlist-filter-ends-with <STRING>
          End with the specified string [aliases: wfe]
      --wordlist-filter-regex <REGEX>
          Match the specified regex [aliases: wfr]
      --wordlist-filter-length <RANGE>
          Length range e.g.: 5, 5-10, 5,10,15, >5, <5 [aliases: wfl]

Response Filtering:
      --filter-status-code <RANGE>   Reponse status code, e.g.: 200, 200-300, 200,300,400, >200, <200 [aliases: fsc]
      --filter-contains <STRING>     Contains the specified string [aliases: fc]
      --filter-starts-with <STRING>  Start with the specified string [aliases: fs]
      --filter-ends-with <STRING>    End with the specified string [aliases: fe]
      --filter-regex <REGEX>         Match the specified regex [aliases: fr]
      --filter-length <RANGE>        Response length e.g.: 100, >100, <100, 100-200, 100,200,300 [aliases: fl]
      --filter-time <RANGE>          Response time range in milliseconds e.g.: >1000, <1000, 1000-2000 [aliases: ft]
```

### Passing parameters as environment variables

You can pass parameters as environment variables. For example, to set the number of threads to `10`:

```bash
THREADS=10 rwalk https://example.com path/to/wordlist.txt
```

is equivalent to:

```bash
rwalk https://example.com path/to/wordlist.txt -t 10
```
The env file located at `~/.config/rwalk/.env` will be loaded automatically.

### Inputting ranges

In some cases , you may want to input a `<RANGE>` of values. 
You can use the following formats:

| Format       | Description                                               |
| :----------- | :-------------------------------------------------------- |
| `5`          | Exactly `5`                                               |
| `5-10`       | Between `5` and `10` (inclusive)                          |
| `5,10`       | Exactly `5` or `10`                                       |
| `>5`         | Greater than `5`                                          |
| `<5`         | Less than `5`                                             |
| `5,10,15`    | Exactly `5`, `10`, or `15`                                |
| `>5,10,15`   | Greater than `5`, or exactly `10` or `15`                 |
| `5-10,15-20` | Between `5` and `10` or between `15` and `20` (inclusive) |

### Response Filtering

To cherry-pick the responses, you can use the `--filter-*` flags to filter specific responses. For example, to only show responses that contain `admin`:

```bash
rwalk https://example.com path/to/wordlist.txt --filter-contains admin
```

or only requests that took more than `1` second:

```bash
rwalk https://example.com path/to/wordlist.txt --filter-time ">1000"
```

Available filters:

- `--filter-starts-with` _`<STRING>`_ or `--fs`
- `--filter-ends-with` _`<STRING>`_ or `--fe`
- `--filter-contains` _`<STRING>`_ or `--fc`
- `--filter-regex` _`<REGEX>`_ or `--fr`
- `--filter-length` _`<LENGTH>`_ or `--fl`
- `--filter-status-code` _`<CODE>`_ or `--fsc`

### Wordlists

You can pass multiple wordlists to `rwalk`. For example:

```bash
rwalk https://example.com path/to/wordlist1.txt path/to/wordlist2.txt
```

`rwalk` will merge the wordlists and remove duplicates. You can also apply filters and transformations to the wordlists (see below).

You can also pass wordlists from stdin:

```bash
cat path/to/wordlist.txt | rwalk https://example.com
```

> [!NOTE]
> A checksum is computed for the wordlists and stored in case you abort the scan. If you resume the scan, `rwalk` will only load the wordlists if the checksums match. See [Saving progress](#saving-and-resuming-scans) for more information.


### Wordlist Filters

You can filter words from the wordlist by using the `--wordlist-filter-*` (`--wf*`) flags. For example, to only use words that start with `admin`:

```bash
rwalk https://example.com path/to/wordlist.txt --wordlist-filter-starts-with admin
```

Available filters:

- `--wordlist-filter-starts-with` _`<STRING>`_ or `--wfs`
- `--wordlist-filter-ends-with` _`<STRING>`_ or `--wfe` 
- `--wordlist-filter-contains` _`<STRING>`_ or `--wfc`
- `--wordlist-filter-regex` _`<REGEX>`_ or `--wfr` 
- `--wordlist-filter-length` _`<LENGTH>`_ or `--wfl` 
- `--wordlist-filter-min-length` _`<LENGTH>`_ or `--wfm`
- `--wordlist-filter-max-length` _`<LENGTH>`_ or `--wfx`


### Wordlist Transformations

To quickly modify the wordlist, you can use the `--transform-*` flags. For example, to add a prefix to all words in the wordlist:

```bash
rwalk https://example.com path/to/wordlist.txt --transform-prefix "."
```

Available transformations:

- `--transform-prefix` _`<PREFIX>`_ or `-P`
- `--transform-suffix` _`<SUFFIX>`_ or `-S`
- `--transform-upper` or `-U`
- `--transform-lower` or `-L`
- `--transform-capitalize` or `-C`

### Interactive mode

You can use the `--interactive` (`-i`) flag to enter interactive mode. In this mode, you can set parameters one by one and run the scan when you're ready.

Available commands:

- `set <PARAM> <VALUE>`: Set a parameter
- `unset <PARAM>`: Unset a parameter
- `list`: Show the current parameters
- `run`: Run the scan
- `exit`: Exit interactive mode
- `help`: Show help
- `clear`: Clear the screen

### Output

By default, `rwalk` will print the results to the terminal. You can also save the results to a file with the `--output` (`-o`) flag:

```bash
rwalk https://example.com path/to/wordlist.txt -o results.json
```

Available output formats:
- `*.json`
- `*.csv`
- `*.md`
- `*.txt`

### Throttling

The throttling value will be multiplied by the number of threads. For example, if you have `10` threads and a throttling value of `5`, the total number of requests per second will be `50`.

### Saving and resuming scans

By default, if you abort the scan with <kbd>Ctrl</kbd> + <kbd>C</kbd>, rwalk will save the progress to a file called `.rwalk.json`. You can resume the scan by running with `--resume`:

```bash
rwalk https://example.com path/to/wordlist.txt --resume
```

If you want to save the progress to a different file, you can use the `--save-file` flag:

```bash
rwalk https://example.com path/to/wordlist.txt --save-file myscan.json 
# or
rwalk https://example.com path/to/wordlist.txt -f myscan.json
```

The auto-saving behavior can be disabled with `--no-save`.

### Proxy support

You can pass a proxy URL with the `--proxy` flag:

```bash
rwalk https://example.com path/to/wordlist.txt --proxy http://pro.xy:8080
```

Authentication is also supported with `--proxy-auth`:

```bash
rwalk https://example.com path/to/wordlist.txt --proxy http://pro.xy:8080 --proxy-auth username:password
```

## Examples

### Basic scan

```bash
rwalk https://example.com path/to/wordlist.txt
```

### Recursive scan

```bash
rwalk https://example.com path/to/wordlist.txt -d 3
```
> **Warning:** Recursive scans can take a long time and generate a lot of traffic. Use with caution.

### Custom headers/cookies

```bash
rwalk https://example.com path/to/wordlist.txt -H "X-Forwarded-For: 203.0.113.195" -c "session=1234567890"
```

### Follow redirects

```bash
rwalk https://example.com path/to/wordlist.txt -F 2
```

### Request throttling

```bash
rwalk https://example.com path/to/wordlist.txt -R 5 -t 10
```

This will send `50` (`5`×`10` threads) requests per second. See [Throttling](#throttling) for more information.

### Custom request body

```bash
rwalk https://example.com path/to/wordlist.txt -m POST -d '{"username": "admin", "password": "admin"}'
```

## FAQ

### Where can I find wordlists?

- [SecLists](https://github.com/danielmiessler/SecLists)
- [DirBuster](https://gitlab.com/kalilinux/packages/dirbuster)
- [OneListForAll](https://github.com/six2dez/OneListForAll)

###

## Benchmarks

The following benchmarks were run on a 2023 MacBook Pro with an M3 Pro chip on a 10 Gbps connection via WiFi. The target was [http://ffuf.me/cd/basic](http://ffuf.me/cd/basic) and the wordlist was [common.txt](https://github.com/danielmiessler/SecLists/blob/master/Discovery/Web-Content/common.txt).

Each tool was run `10` times with `100` threads. The results are below:

| Command     |       Mean [s] | Min [s] | Max [s] |    Relative |
| :---------- | -------------: | ------: | ------: | ----------: |
| `rwalk`     |  6.068 ± 0.146 |   5.869 |   6.318 | 1.15 ± 0.03 |
| `dirsearch` | 14.263 ± 0.250 |  13.861 |  14.719 | 2.70 ± 0.07 |
| `ffuf`      |  5.285 ± 0.090 |   5.154 |   5.358 |        1.00 |

## License

Licensed under the [MIT License](LICENSE).
