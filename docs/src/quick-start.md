# Quick Start

A quick guide to get you started with `rwalk`. This tool's philosophy is to provide a simple and fast way to scan websites for files and directories.

## Installation

The easiest way to install `rwalk` is to use the pre-built binaries. You can also install it using `cargo`, the Rust package manager.
The pre-built binaries are available for macOS, Linux and Windows and can be downloaded from the [releases page](https://github.com/cestef/rwalk/releases).

### Using homebrew (recommended, macOS and Linux only)

```bash
brew install cestef/tap/rwalk
```

### Using cargo

```bash
cargo install rwalk
```

or with `cargo-binstall`:

```bash
cargo binstall rwalk
```

This will directly download the binary from the latest release and install it in `~/.cargo/bin`.

### From source

```bash
git clone https://github.com/cestef/rwalk
cd rwalk
cargo install --path .
```

## Usage

#### Modes

The core concept of `rwalk` revolves around different **scanning modes**. Each of these modes is designed to provide a different way to scan a website. The available modes are:

- [`recursive`](./modes.md#recursive): Start from a given path and check each of its subdirectories
- [`classic`](./modes.md#classic): Standard Fuzzing mode, where you provide a list of patterns to check
- [`spider`](./modes.md#spider): Start from a given path and follow all links found until a certain depth

The mode can be specified using the `--mode` (`-m`) option. If not specified, the mode will be automatically detected based on the provided arguments. To read more about the modes, check the [modes documentation](./modes.md).

#### Basic usage

To get a list of all available options, you can run:

```bash
rwalk --help
```

A markdown version of the help message is also available [here](./options.md).

The basic syntax for running `rwalk` is as follows:

```bash
rwalk [OPTIONS] [URL] [FILE:KEY]...
```

Where:

- `[URL]` is the target URL (`http://example.com`)
- `[FILE:KEY]` are the wordlists to use for fuzzing. Each wordlist is identified by an optional key, which is used to reference it in some options. (`/path/to/wordlist:KEY`)
- `[OPTIONS]` are the various options that can be used to customize the scan. 

#### Examples

In these examples, we will use the [`onelistforallmicro.txt`](https://raw.githubusercontent.com/six2dez/OneListForAll/main/onelistforallmicro.txt).
You can download it using `curl`:

```bash
curl https://raw.githubusercontent.com/six2dez/OneListForAll/main/onelistforallmicro.txt -o common.txt
```

In most of our examples, [ffuf.me](http://ffuf.me) will be used as the target URL. A huge thanks to [BuildHackSecure](https://github.com/BuildHackSecure/ffufme) for providing this service.

##### Recursive mode

```
rwalk http://ffuf.me/cd/recursion common.txt -d 3
```

```ansi
[0;32m✓[0m [0;2m200[0m /cd/recursion ([0;2mdir[0m)[0m
[0;2m└─ [0;38:2:1:255:165:0m✖[0m [0;2m403[0m /admin ([0;2mdir[0m)[0m
[0;2m   └─ [0;38:2:1:255:165:0m✖[0m [0;2m403[0m /users ([0;2mdir[0m)[0m
[0;2m      └─ [0;32m✓[0m [0;2m200[0m /96 ([0;2mtext/html[0m)[0m
```