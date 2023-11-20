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
- [ ] Proxy support
- [ ] Request throttling

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
  -t, --threads <THREADS>
          Number of threads to use
  -d, --depth <DEPTH>
          Maximum depth to crawl [default: 1]
  -o, --output <OUTPUT>
          Output file
  -T, --timeout <TIMEOUT>
          Request timeout in seconds [default: 10]
  -u, --user-agent <USER_AGENT>
          User agent
  -q, --quiet
          Quiet mode
  -m, --method <METHOD>
          HTTP method [default: GET]
  -d, --data <DATA>
          Data to send with the request
  -H, --headers <key:value>
          Headers to send
  -c, --cookies <key=value>
          Cookies to send
  -I, --case-insensitive
          Case insensitive
  -F, --follow-redirects <FOLLOW_REDIRECTS>
          Follow redirects [default: 0]
  -R, --throttle <THROTTLE>
          Request throttling (requests per second) per thread [default: 0]
  -h, --help
          Print help
  -V, --version
          Print version
```

> **Note:** The throttling value will be multiplied by the number of threads. For example, if you have `10` threads and a throttling value of `5`, the total number of requests per second will be `50`.

## Benchmarks

The following benchmarks were run on a 2023 MacBook Pro with an M3 Pro chip on a 10 Gbps connection via WiFi. The target was [http://ffuf.me/cd/basic](http://ffuf.me/cd/basic) and the wordlist was [common.txt](https://github.com/danielmiessler/SecLists/blob/master/Discovery/Web-Content/common.txt).

Each tool was run `10` times with `100` threads. The results are below:

| Command                                                            |       Mean [s] | Min [s] | Max [s] |    Relative |
| :----------------------------------------------------------------- | -------------: | ------: | ------: | ----------: |
| `rwalk https://google.com ~/Downloads/common.txt -t 100`           |  6.068 ± 0.146 |   5.869 |   6.318 | 1.15 ± 0.03 |
| `dirsearch -u https://google.com -w ~/Downloads/common.txt -t 100` | 14.263 ± 0.250 |  13.861 |  14.719 | 2.70 ± 0.07 |
| `ffuf -w ~/Downloads/common.txt -u https://google.com/FUZZ -t 100` |  5.285 ± 0.090 |   5.154 |   5.358 |        1.00 |

[ffuf](https://github.com/ffuf/ffuf) is the fastest tool... but not by much. rwalk is only `1.15x` slower than ffuf and ~`2.5x` faster than dirsearch. Not bad for a first release!

## License

Licensed under the [MIT License](LICENSE).
