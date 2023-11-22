<p align="center">
    <img src="assets/header.png" alt="rwalk" />
</p>

A blazing fast web directory scanner written in Rust. It's like [dirsearch](https://github.com/maurosoria/dirsearch) but faster and with less features.

## Features

- [x] Multi-threaded
- [x] Recursive directory scanning
- [x] Custom wordlists (merge multiple wordlists, filter out words, etc.) 
- [x] Write results to file (JSON, CSV, etc.)
- [x] Configurable request parameters (headers, cookies, etc.)
- [x] Save progress to resume later
- [x] Request throttling
- [ ] Proxy support

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

Filtering:
  -F, --filter-contains <STRING>     Contains the specified string
      --filter-starts-with <STRING>  Start with the specified string
      --filter-ends-with <STRING>    End with the specified string
      --filter-regex <REGEX>         Filter out words that match the specified regex
      --filter-max-length <LENGTH>   Maximum length
      --filter-min-length <LENGTH>   Minimum length
      --filter-length <LENGTH>       Exact length
```

### Wordlists

You can pass multiple wordlists to rwalk. For example:

```bash
rwalk https://example.com path/to/wordlist1.txt path/to/wordlist2.txt
```

rwalk will merge the wordlists and remove duplicates. You can also apply filters and transformations to the wordlists (see below).

> **Note:** A checksum is computed for the wordlists and stored in case you abort the scan. If you resume the scan, rwalk will only load the wordlists if the checksums match. See [Saving progress](#saving-and-resuming-scans) for more information.

### Filters

You can filter out words from the wordlist by using the `--filter-*` flags. For example, to filter out all words that start with `admin`:

```bash
rwalk https://example.com path/to/wordlist.txt --filter-starts-with admin
```

Available filters:

- `--filter-starts-with` _`<STRING>`_
- `--filter-ends-with` _`<STRING>`_
- `--filter-contains` _`<STRING>`_
- `--filter-regex` _`<REGEX>`_
- `--filter-length` _`<LENGTH>`_
- `--filter-min-length` _`<LENGTH>`_
- `--filter-max-length` _`<LENGTH>`_


### Transformations

To quickly modify the wordlist, you can use the `--transform-*` flags. For example, to add a prefix to all words in the wordlist:

```bash
rwalk https://example.com path/to/wordlist.txt --transform-prefix "."
```

Available transformations:

- `--transform-prefix` _`<PREFIX>`_
- `--transform-suffix` _`<SUFFIX>`_
- `--transform-upper`
- `--transform-lower`
- `--transform-capitalize`

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
rwalk https://example.com path/to/wordlist.txt -F myscan.json
```

The auto-saving behavior can be disabled with `--no-save`.

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

[ffuf](https://github.com/ffuf/ffuf) is the fastest tool... but not by much. rwalk is only `1.15x` slower than ffuf and ~`2.5x` faster than dirsearch. Not bad for a first release!

## License

Licensed under the [MIT License](LICENSE).
