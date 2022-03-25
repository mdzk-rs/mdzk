---
title: mdzk documentation
---

This document will go through how mdzk operates and how you can use it in your workflows. Currently, it is quite limited, as mdzk is still under heavy development. If you have any suggestions to things that should be added, [open an issue on our GitHub](https://github.com/mdzk-rs/mdzk/issues/new) describing it and mark it with the `documentation` label. Any contributions are of course welcome.

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
- **Notes** are the nodes in a vault.
- **Internal links** create edges connecting each note.

In the filesystem, a vault is simply a directory and the notes are Markdown-files. mdzk takes a directory as input, recursively finds any Markdown-files in it and loads each of them as a node in the graph. It then parses every file and establishes edges between the nodes based on the links contained in it.

The *vault* data structure is a very useful concept in dealing with notes and their relation together. Finding backlinks is now as simple as following every incoming edge and see where it leads you.
