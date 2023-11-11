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
- [ ] Save progress to resume later
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
Usage: rwalk [OPTIONS] <HOST> <WORDLISTS>...

Arguments:
  <HOST>          Host to scan
  <WORDLISTS>...  Path(s) to wordlist(s)

Options:
  -p, --port <PORT>                  Optional port to scan
  -v, --verbose...                   Turn verbose information on
  -t, --threads <THREADS>            The number of threads to use Defaults to the number of logical cores * 10 [default: 0]
  -T, --throttle <THROTTLE>          Throttle requests to the host (in milliseconds) [default: 0]
  -R, --retries <RETRIES>            The number of retries to make [default: 0]
  -r, --redirects <REDIRECTS>        The number of redirects to follow [default: 0]
  -u, --user-agent <USER_AGENT>      The user agent to use
  -d, --depth <DEPTH>                Maximum depth to crawl [default: 0]
  -t, --timeout <TIMEOUT>            Timeout for requests (in seconds) [default: 5]
  -o, --output <OUTPUT>              The output file to write to
  -m, --method <METHOD>              Method to test against Possible values: GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, TRACE, PATCH [default: GET]
  -c, --content-type <CONTENT_TYPE>  The content type to use [default: text/plain]
  -d, --data <DATA>                  Optional data to send with the request
  -I, --case-insensitive             Whether or not to crawl case insensitive
  -h, --help                         Print help
  -V, --version                      Print version
```

## License

Licensed under the [MIT License](LICENSE).
