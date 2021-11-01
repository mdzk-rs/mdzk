<h1 align="center">mdzk</h1>

<p align="center">A lovingly designed system and static publishing tool for your plain text Zettelkasten</p>

mdzk is a plain text Zettelkasten system that is based on the mdBook API. It consists of specialized preprocessors for handling key features such as wikilinks, backlinks and front matter, all tied together in the mdzk CLI. mdzk allows you to use whatever text editor you prefer - be it Obsidian, Vim, Visual Studio Code or even Notepad. You can view your notes with live updating as you edit them, have them automatically published to GitHub Pages on push and much more.

## Installation

### Cargo

You can install mdzk with Cargo by running

```
$ cargo install mdzk
```

### Homebrew

mdzk is available in [Homebrew](https://brew.sh/). Install by running

```
$ brew install mdzk
```

### Nix

You can use without installing with [Nix](https://nixos.org). **Note:** This will compile it from source and make it available from your nix store.

```
$ nix run github:mdzk-rs/mdzk help
```

### Pre-compiled binaries

On the [Releases](https://github.com/mdzk-rs/mdzk/releases) page, you can find pre-compiled binaries for Windows and Linux (x86).

## Getting started

We are still figuring out the exact user interface, so there would not be much help in a "getting started" guide just yet. mdzk works very much like [mdBook](https://rust-lang.github.io/mdBook/cli/index.html), so you can use their guide for the time being. We hope to provide a full documentation very soon...

## Important notes

- **Supported characters**:

    These characters are disallowed in note filenames: `=`, <code>\`</code>, `^`, `#`, `|`, `:`, `/`, `[` and `]`.
