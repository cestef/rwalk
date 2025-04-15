+++
weight = 0
title = "Get Started"
+++

## Installation

`rwalk` is available on multiple platforms and package managers. You can install it using any of the following methods:

### From source

```bash, copy
cargo install --git "https://github.com/cestef/rwalk.git" --branch main
```

### From crates.io

```bash, copy
cargo install rwalk
# or
cargo binstall rwalk
```

### With brew

```bash, copy
brew install cestef/tap/rwalk
```

### From Nix

```bash, copy
# without flakes:
nix-env -iA nixpkgs.rwalk
# with flakes:
nix profile install nixpkgs#rwalk
```

### From AUR

```bash, copy
paru -S rwalk
# or
yay -S rwalk
```

### From prebuilt binaries

Grab them from the [latest release](https://github.com/cestef/rwalk/releases/latest) page.

## Usage

```bash, copy
rwalk [OPTIONS] <URL> <WORDLISTS>...
```

See `rwalk --help` for more information.
