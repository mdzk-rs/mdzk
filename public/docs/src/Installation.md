You can install mdzk from multiple package managers:

| Source | Installation command |
| --------------: | :------------------- |
| [AUR](https://aur.archlinux.org/packages/mdzk/) (with [Paru](https://github.com/Morganamilo/paru)) | `paru -S mdzk` |
| [Crates.io](https://crates.io/crates/mdzk) | `cargo install mdzk` |
| [Homebrew](https://formulae.brew.sh/formula/mdzk#default) | `brew install mdzk` |
| [Nix](https://nixos.org) | `nix shell nixpkgs#mdzk` or `nix run nixpkgs#mdzk -- <command>` |

There is also a range of pre-built binaries available through the [release page on GitHub](https://github.com/mdzk-rs/mdzk/releases).

## Run mdzk main branch through Nix

mdzk also provides a [Nix Flake](https://nixos.wiki/wiki/Flakes) file, which means you can use [Nix](https://nixos.org/) to run the main branch of mdzk without waiting the next release. For example, to initialize an mdzk inside the current directory, you can run:

```
$ nix run github:mdzk-rs/mdzk init
```

## Build mdzk yourself

If you want the latest and greatest, you can build mdzk from scratch by cloning the repo and building using [Rust tooling](https://www.rust-lang.org/tools/install):

```
$ git clone https://github.com/mdzk-rs/mdzk.git
$ cd mdzk
$ cargo build --release
```

An mdzk binary for your system will now be available in `./target/release`.
