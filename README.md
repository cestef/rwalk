<p align="center">
    <img src="assets/header.png" alt="rwalk" />
</p>

<p align="center">
    <a href="https://crates.io/crates/rwalk" style="text-decoration: none;">
        <img src="https://img.shields.io/github/actions/workflow/status/cestef/rwalk/release.yml?labelColor=%231e1e1e&color=%231e1e1e" alt="Crates.io" />
    </a>
    <a href="https://img.shields.io/github/v/release/cestef/rwalk?labelColor=%231e1e1e&color=%231e1e1e" style="text-decoration: none;">
        <img src="https://img.shields.io/github/v/release/cestef/rwalk?labelColor=%231e1e1e&color=%231e1e1e" alt="Release" />
    </a>
    <a href="LICENSE" style="text-decoration: none;">
        <img src="https://img.shields.io/github/license/cestef/rwalk?labelColor=%231e1e1e&color=%231e1e1e" alt="License" />
    </a>
</p>

> [!WARNING]
> This is the development branch. I am mostly testing new features here. If you want to use a stable version, please check the [main branch](https://github.com/cestef/rwalk/tree/main).

A blazingly fast web directory scanner written in Rust. It's like [dirsearch](https://github.com/maurosoria/dirsearch) but on steroids.
It is designed to be fast in [**recursive scans**](https://rwalk.cstef.dev/docs/modes) and to be able to handle large wordlists.

Unlike other tools, rwalk does **<u>not</u>** provide advanced fuzzing features such as **parameter fuzzing**, **header discovery**, etc.

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

### From [AUR](https://aur.archlinux.org/packages/rwalk) <!-- omit in toc -->

```bash
paru -S rwalk
```

<small>
    <p align="center">
        <i>You can also download the latest binary from the <a href="https://github.com/cestef/rwalk/releases/latest">releases page</a>.</i>
    </p>
</small>


## Contributing

_Contributions are welcome! I am always looking for new ideas and improvements._

If you want to contribute to rwalk, please read the [CONTRIBUTING.md](CONTRIBUTING.md) file.

Make sure that your commits follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) standard.
This project uses [commitizen](https://commitizen-tools.github.io/commitizen/) to help you with that.

## License

Licensed under the [MIT License](LICENSE).