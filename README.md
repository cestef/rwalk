<p align="center">
    <img src="assets/header.png" alt="rwalk" />
</p>

A blazing fast web directory scanner written in Rust. It's like [dirsearch](
    https://github.com/maurosoria/dirsearch) but faster and with less features.

## Features

- [x] Multi-threaded
- [x] Recursive directory scanning
- [x] Custom wordlists (merge multiple wordlists, filter out words, etc.) 
- [x] Write results to file
- [x] Configurable request parameters (headers, cookies, etc.)
- [x] Save progress to resume later
- [ ] Proxy support

## Installation

### From source

```bash
git clone https://github.com/cestef/rwalk.git
cd rwalk
```

## Running

**With [just](https://github.com/casey/just)**

```bash
just run https://example.com path/to/wordlist.txt
```

**With [cargo](
    https://doc.rust-lang.org/cargo/getting-started/installation.html)**

```bash
cargo run --release -- https://example.com path/to/wordlist.txt
```

## Usage

```text
Usage: rwalk [OPTIONS] <URL> <WORDLISTS>...

Arguments:
  <URL>           Target URL
  <WORDLISTS>...  Wordlist(s)

Options:
  -t, --threads <THREADS>                    Number of threads to use
  -d, --depth <DEPTH>                        Maximum depth to crawl [default: 1]
  -o, --output <OUTPUT>                      Output file
  -T, --timeout <TIMEOUT>                    Request timeout in seconds [default: 5]
  -u, --user-agent <USER_AGENT>              User agent
  -q, --quiet                                Quiet mode
  -m, --method <METHOD>                      HTTP method [default: GET]
  -d, --data <DATA>                          Data to send
  -H, --headers <key:value>                  Headers to send
  -c, --cookies <key=value>                  Cookies to send
  -I, --case-insensitive                     Case insensitive
  -F, --follow-redirects <FOLLOW_REDIRECTS>  Follow redirects [default: 0]
  -h, --help                                 Print help
  -V, --version                              Print version
```

## License

Licensed under the [MIT License](LICENSE).
