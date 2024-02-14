
<p align="center">
    <img src="assets/header.png" alt="rwalk" />
</p>

[![Crates.io](https://img.shields.io/crates/v/rwalk)](https://crates.io/crates/rwalk)
[![GitHub](https://img.shields.io/github/license/cestef/rwalk)](LICENSE)
[![Release](https://img.shields.io/github/v/release/cestef/rwalk)](https://github.com/cestef/rwalk/releases/latest)


A blazing fast web directory scanner written in Rust. It's like [dirsearch](https://github.com/maurosoria/dirsearch) but faster and with less features.
It is designed to be fast in [**recursive scans**](#recursive-scan) and to be able to handle large wordlists. 

Unlike other tools, rwalk does **<u>not</u>** provide advanced fuzzing features such as **parameter fuzzing**, **header discovery**, etc.

<p align="center">
    <img src="assets/rwalk.gif" width="auto" height="300px">
</p>

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

## Installation

### From [homebrew](https://brew.sh)

```bash
brew install cestef/tap/rwalk
```

### From [crates.io](https://crates.io/crates/rwalk)

```bash
cargo install rwalk
```

### From source

```bash
git clone https://github.com/cestef/rwalk.git
cd rwalk
cargo install --path .
```

<small>
    <p align="center">
        <i>You can also download the latest binary from the <a href="https://github.com/cestef/rwalk/releases/latest">releases page</a>.</i>
    </p>
</small>


## Development

**With [just](https://github.com/casey/just)**

```bash
just run https://example.com path/to/wordlist.txt
```

**With [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)**

```bash
cargo run --release -- https://example.com path/to/wordlist.txt
```

## Usage

You can run `rwalk --help` or [read the help file](HELP.md) for more information.


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

To cherry-pick the responses, you can use the `--filter` (`-f`) flags to filter specific responses. For example, to only show responses that contain `admin`:

```bash
rwalk https://example.com path/to/wordlist.txt --filter contains:admin
```

or only requests that took more than `1` second:

```bash
rwalk https://example.com path/to/wordlist.txt --filter "time:\\>1000"
```

Available filters:

- `starts`: _`<STRING>`_ 
- `ends`: _`<STRING>`_
- `contains`: _`<STRING>`_
- `regex`: _`<REGEX>`_
- `length`: _`<RANGE>`_
- `status`: _`<RANGE>`_
- `time`: _`<RANGE>`_
- `hash`: _`<STRING>`_ (MD5)

**Note:** Each filter can be negated by adding a `!` before the filter. For example, to exclude responses that contain `admin`:

```bash
rwalk https://example.com path/to/wordlist.txt --filter "!contains:admin"
```

### Wordlists

You can pass multiple wordlists to `rwalk`. For example:

```bash
rwalk https://example.com path/to/wordlist1.txt path/to/wordlist2.txt
```

`rwalk` will merge the wordlists and remove duplicates. You can also apply filters and transformations to the wordlists (see below).

You can also pass wordlists from stdin:

```bash
cat path/to/wordlist.txt | rwalk https://example.com -
```

> [!NOTE]
> A checksum is computed for the wordlists and stored in case you abort the scan. If you resume the scan, `rwalk` will only load the wordlists if the checksums match. See [Saving progress](#saving-and-resuming-scans) for more information.


### Wordlist Filters

You can filter words from the wordlist by using the `--wordlist-filter` (`-w`) flag. For example, to only use words that start with `admin`:

```bash
rwalk https://example.com path/to/wordlist.txt --wordlist-filter starts:admin
```

Available filters:

- `starts`: _`<STRING>`_
- `ends`: _`<STRING>`_ 
- `contains`: _`<STRING>`_ 
- `regex`: _`<REGEX>`_
- `length`: _`<RANGE>`_


### Wordlist Transformations

To quickly modify the wordlist, you can use the `--transform` flag. For example, to add a suffix to all words in the wordlist:

```bash
rwalk https://example.com path/to/wordlist.txt --transform suffix:.php
```

To replace all occurrences of `admin` with `administrator`:

```bash
rwalk https://example.com path/to/wordlist.txt --transform replace:admin=administrator
```

Available transformations:

- `prefix`: _`<STRING>`_
- `suffix`: _`<SUFFIX>`_
- `remove`: _`<STRING>`_
- `replace`: _`<OLD=NEW>`_
- `upper`
- `lower`
- `capitalize`
- `reverse`

### Additional response details

If you need more details about the matched responses, you can use the `--show` flag. For example, to show the body hash and length:

```bash
rwalk https://example.com path/to/wordlist.txt --show hash --show length 
```

Available details:

- `length`
- `hash`
- `headers`
- `body`
- `headers_length`
- `headers_hash`

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


```bash
rwalk https://example.com path/to/wordlist.txt --throttle 5 -t 10 
```

### Saving and resuming scans

By default, if you abort the scan with <kbd>Ctrl</kbd> + <kbd>C</kbd>, rwalk will save the progress to a file called `.rwalk.json`. You can resume the scan by running with `--resume`:

```bash
rwalk https://example.com path/to/wordlist.txt --resume
```

If you want to save the progress to a different file, you can use the `--save-file` flag:

```bash
rwalk https://example.com path/to/wordlist.txt --save-file myscan.json 
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
rwalk https://example.com path/to/wordlist.txt -R 2
```

### Custom request body

```bash
rwalk https://example.com path/to/wordlist.txt -m POST -D '{"username": "admin", "password": "admin"}'
```

## FAQ

### Where can I find wordlists?

- [SecLists](https://github.com/danielmiessler/SecLists)
- [DirBuster](https://gitlab.com/kalilinux/packages/dirbuster)
- [OneListForAll](https://github.com/six2dez/OneListForAll)

### How do I get support?

Open an issue or ask in the [Discord server](https://cstef.dev/discord). 

### Is rwalk stable?

rwalk is stable but it's still in the early stages of development. It should work for most use cases but there may be bugs.

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
