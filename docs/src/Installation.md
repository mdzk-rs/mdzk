You can install mdzk from multiple package managers:

| Source | Installation command |
| --------------: | :------------------- |
| [Crates.io](https://crates.io/crates/mdzk) | `cargo install mdzk` |
| [Homebrew](https://formulae.brew.sh/formula/mdzk#default) | `brew install mdzk` |
| [AUR](https://aur.archlinux.org/packages/mdzk/) (with [Paru](https://github.com/Morganamilo/paru)) | `paru -S mdzk` |

There is also a range of pre-built binaries available through the [release page on GitHub](https://github.com/mdzk-rs/mdzk/releases).

## Run mdzk through Nix

mdzk is available in [Nix unstable](https://search.nixos.org/packages?channel=unstable&show=mdzk&from=0&size=50&sort=relevance&type=packages&query=mdzk), which means you can use [Nix](https://nixos.org/) to run all the functionality of mdzk without really "installing" it. For example, to initialize an mdzk inside the current directory, you can run:

<!-- FIXME(ratsclub): Change the below command to fetch mdzk from nixpkgs.unstable? -->

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
