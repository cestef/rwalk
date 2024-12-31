<p align="center">
    <img src="assets/header.png" alt="rwalk" />
</p>

[![Crates.io](https://img.shields.io/crates/v/rwalk)](https://crates.io/crates/rwalk)
[![GitHub](https://img.shields.io/github/license/cestef/rwalk)](LICENSE)
[![Release](https://img.shields.io/github/v/release/cestef/rwalk)](https://github.com/cestef/rwalk/releases/latest)

A blazingly fast web directory scanner written in Rust. It's like [dirsearch](https://github.com/maurosoria/dirsearch) but on steroids.
It is designed to be fast in [**recursive scans**](https://rwalk.cstef.dev/docs/modes) and to be able to handle large wordlists.

Unlike other tools, rwalk does **<u>not</u>** provide advanced fuzzing features such as **parameter fuzzing**, **header discovery**, etc.

<p align="center">
    <img src="assets/rwalk.gif">
</p>


## Quick Installation

### On [Nix](https://nixos.org)
```bash
# without flakes:
nix-env -iA nixpkgs.rwalk
# with flakes:
nix profile install nixpkgs#rwalk
```

### From [homebrew](https://brew.sh) <!-- omit in toc -->

```bash
brew install cestef/tap/rwalk
```

### With [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) <!-- omit in toc -->

```bash
cargo binstall rwalk
```

### From [crates.io](https://crates.io/crates/rwalk) <!-- omit in toc -->

```bash
cargo install rwalk
```

<small>
    <p align="center">
        <i>You can also download the latest binary from the <a href="https://github.com/cestef/rwalk/releases/latest">releases page</a>.</i>
    </p>
</small>

## Documentation

The full documentation can be found at **[https://rwalk.cstef.dev](https://rwalk.cstef.dev)**.

## Task Runner

This project uses [`braisé`](https://github.com/cestef/braise) as a task runner. You can find all the available tasks in the [`braise.toml`](braise.toml) file.

## Benchmarks

The following benchmarks were run on a 2023 MacBook Pro with an M3 Pro chip on a 10 Gbps connection via WiFi. The target was [http://ffuf.me/cd/basic](http://ffuf.me/cd/basic) and the wordlist was [common.txt](https://github.com/danielmiessler/SecLists/blob/master/Discovery/Web-Content/common.txt).

Each tool was run `10` times with `100` threads. The results are below:

| Command     |      Mean [s] | Min [s] | Max [s] |    Relative |
| :---------- | ------------: | ------: | ------: | ----------: |
| `rwalk`     | 2.406 ± 0.094 |   2.273 |   2.539 |        1.00 |
| `dirsearch` | 8.528 ± 0.149 |   8.278 |   8.743 | 3.54 ± 0.15 |
| `ffuf`      | 2.552 ± 0.181 |   2.380 |   3.005 | 1.06 ± 0.09 |

If you want to run the benchmarks yourself, you can use the `bench` command:

```bash
br bench
```

Arguments can also be passed to the `bench` command:

```bash
URL="http://ffuf.me/cd/basic" br bench
```

Please take these results with a grain of salt.

> <i> "There are three types of lies: lies, damned lies and benchmarks"</i>

## Contributing

_Contributions are welcome! I am always looking for new ideas and improvements._

If you want to contribute to rwalk, please read the [CONTRIBUTING.md](CONTRIBUTING.md) file.

Make sure that your commits follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) standard.
This project uses [commitizen](https://commitizen-tools.github.io/commitizen/) to help you with that.

## License

Licensed under the [MIT License](LICENSE).
