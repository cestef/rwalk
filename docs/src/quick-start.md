# Quick Start

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

