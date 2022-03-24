---
title: mdzk documentation
---

**mdzk** is a plain text Zettelkasten system that works as the backend to your connected notes. It can take any folder of Markdown-files and process it into a directed graph that you can use to produce rich workflows, integrate with your favorite static site generator, host a language server and much more.


---


# Installing {#installing}

You can install mdzk from multiple package managers:

| Source | Installation command |
| -------------- | -------------------: |
| [AUR](https://aur.archlinux.org/packages/mdzk/) (with [Paru](https://github.com/Morganamilo/paru)) | `paru -S mdzk` |
| [Crates.io](https://crates.io/crates/mdzk) | `cargo install mdzk` |
| [Homebrew](https://formulae.brew.sh/formula/mdzk#default) | `brew install mdzk` |
| [Nix](https://search.nixos.org/packages?channel=unstable&show=mdzk&from=0&size=50&sort=relevance&type=packages&query=mdzk) | `nix run nixpkgs#mdzk -- <command>` |

There is also a range of pre-built binaries available through the [release page on GitHub](https://github.com/mdzk-rs/mdzk/releases).

## Build mdzk yourself

If you want the latest and greatest, you can build mdzk from scratch by cloning the repo and building using [Rust tooling](https://www.rust-lang.org/tools/install):

```shell
$ git clone https://github.com/mdzk-rs/mdzk.git
$ cd mdzk
$ cargo build --release
```

An mdzk binary for your system will now be available in `./target/release`.


---


# Core concepts

At it's core, mdzk operates on a *vault* with *notes* containing optional *internal links*. Let's unpack these terms:

- A **vault** is a [directed graph](https://en.wikipedia.org/wiki/Directed_graph).
- The vertices in a vault are called **notes**.
- **Internal links** create edges connecting each note.

In the filesystem, a vault is simply a directory and the notes are Markdown-files.
