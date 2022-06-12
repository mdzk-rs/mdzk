---
title: mdzk documentation
---

This document will go through how mdzk operates and how you can use it in your workflows. Currently, it is quite limited, as mdzk is still under heavy development. If you have any suggestions to things that should be added, [open an issue on our GitHub](https://github.com/mdzk-rs/mdzk/issues/new) describing it and mark it with the `documentation` label. Any contributions are of course welcome.

---


# Installing

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


# Getting started

Getting started with mdzk is very simple. You only need a directory of Markdown files, linked together with `[[wiki style]]` links (you can read more about the syntax [here](#arc)). From a terminal, you can then run

```shell
$ mdzk
```

inside the directory in question. This will spit out a bunch of JSON to the terminal. This JSON represents your vault, it's structure, and the notes it contains along with their metadata. The goal of mdzk is to allow interoperability by piping the JSON into other tools which can process it into something useful. As an example, this one-liner will produce data that lets you visualize your vault with [D3.js](https://d3js.org/):

```shell
$ <one-liner here>
```

## Static site generation

### Publishing with GitHub Pages

## Language server and tree-sitter


# Core concepts

At it's core, mdzk creates a *vault* of *notes* containing *arcs*. Let's unpack these terms:

- A **vault** is a [simple directed graph](https://en.wikipedia.org/wiki/Directed_graph).
- **Notes** are the nodes in a vault.
- An **arc** is a directed edge connecting two notes.

## Vault

Mathematically we can define a vault as the tuple $V = (N, A)$, where

- $N = \{n_i\}$ is a set of [notes](#note);
- $A \subseteq \{(n_i, n_j)$ where $(n_i, n_j)\in N^2\}$ is a set of [arcs](#arc).[^arcs-def]

mdzk represents a vault in memory as an [adjacency matrix](https://mathworld.wolfram.com/AdjacencyMatrix.html). This gives us very fast lookup times on arcs, and simplifies finding inversed arcs ([backlinks](https://en.wikipedia.org/wiki/Backlink)).

In the filesystem, a vault is simply a directory with a bunch of Markdown files. You may structure this directory however you want and include any other filetypes - mdzk does not care. Something to keep in mind: mdzk does not have any notion of a directory hierarchy. You may bury a note in a subdirectory of a subdirectory of a subdirectory if you want, mdzk will still handle it the same way as a note placed in the root.

## Note

A note is a node in a vault, represented by a single Markdown file inside the vault's source directory. mdzk recursively finds all Markdown files in a directory and creates notes from them. Notes also contain some metadata:

- An [ID](#note-ids);
- The original content of the source file;
- The processed content of the source file;
- A relative path from the vault root;
- An optional date;
- Optional tags.

Since mdzk only processes front matter and links, there is no limitation on what Markdown extensions you can use in a note's content. However, mdzk adheres to [CommonMark](https://commonmark.org/), so links inside e.g. code blocks and raw HTML will not get touched. If you are unfamiliar with CommonMark, I strongly recommend you check out [this quick guide](https://commonmark.org/help/) to get the gist of it, and stick to it's conventions where possible. We also support [tables](https://github.github.com/gfm/#tables-extension-), [task lists](https://github.github.com/gfm/#task-list-items-extension-), [strikethrough](https://github.github.com/gfm/#strikethrough-extension-) and [footnotes](https://talk.commonmark.org/t/how-should-footnotes-behave/1106).

### Note IDs

Each note is uniquely identified by a 16-digit hexidecimal ID (64 bits). This ID is gained from the [Fowler-Noll-Vo hash](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function) of the relative path from the vault root to the note's source file. This means that you can change the title, metadata and content of your note as much as you want without changing it's ID - as long as the filepath remains the same.

As a user, you will likely never need to care about a note's ID; it is only a way for mdzk and external programs to handle uniqueness in cases like name changes or note's with the same title.

## Arc

An arc is created by writing a [wiki-style](https://www.mediawiki.org/wiki/Help:Links#Internal_links) link inside a note, namely by surrounding a note identifier by double square brackets like this: `[[link to a note]]`. The note identifier can be both the title of a note or a path.

---

Consider the vault with the following tree:

```shell
.
├── sub
│  ├── subsub
│  │  └── note-0.md
│  └── note-1.md
└── note-2.md
```

Producing a visualization of this vault, might look like this:

<!-- Using http://bl.ocks.org/rkirsling/5001347 to produce illustrative graphs atm. -->
![](images/vaults_01.png){ width=50% }

As you can see, the paths were totally ignored; *[arcs](#arc) define the vault structure, not filepaths*. Paths are still stored as metadata, so custom workflows using directory hierarchies can still be made if one wants.


---


# Vault JSON representation

```json
{
  "notes": [
    {
      // A unique ID, formatted as a hex digit
      "id": string,

      // The given title, taken from the filename or
      // explicitly set via front matter
      "title": string,

      // A relative path from the vault root to the note
      "path": string | null,

      // A set of tags set for the note
      "tags": [string],

      // RFC3339-formatted datestring
      "date": string | null,

      // The original content of the source file that
      // produced this note
      "original-content": string,

      // The content of the note. All internal links are
      // converted to CommonMark and the front matter
      // is stripped
      "content": string,

      "arcs": {
          // An array of note IDs that are the
          // destination of an arc starting at the
          // current note
          "outgoing": [string],
          // An array of note IDs where the current
          // note is the destination
          "incoming": [string]
      },
    }
  ],
  // Not yet implemented: "tags": []
}
```

[^arcs-def]: Note that this definition permits loops, meaning notes can link to themselves. This does not provide any real practical benefits, but it simplifies how we model the vault and gives some flexibility in linkage.
