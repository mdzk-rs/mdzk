<h1 align="center">mdzk</h1>

<p align="center">Early stage prototype of a plain text Zettelkasten system and static site generator based on the mdBook API.</p>

- [Features](#features)
  - [Summary generator](#summary-generator)
  - [Wikilinks](#wikilinks)
  - [Backlinks](#backlinks)
  
All sections below serve as conversation starters and general questions on major design choices of this program.

# Features

## `SUMMARY.md` generator

- Will it be generated with a hierarchy based on linkage or directory structure, or should we force it to be flat?
- Note titles are determined in the summary, which in a large vault will get cumbersome to maintain. We could include the option of a TOML front matter in each note where a title can be set (like Neuron). Alternatively, the CLI could have a command for changing titles as well (with fuzzy search and all that).

This file is actually quite clever, as it serves as an index of the ZK that could easily be parsed by external programs. One use case I've been thinking of is a Neovim plugin for searching the vault with [Telescope](https://github.com/nvim-telescope/telescope.nvim). Here, a simple line by line regex could feed both the file names and the corresponding titles to Telescope, making editor search really simple to implement.

## Wikilinks

I guess we could just replace it to the commonmark equivalent as we were already doing before.

## Backlinks

1. Will it show an overview of the paragraph? Something like:

```markdown
# Note

I'm linking [[othernote]] here.
```

## Network Chart

Hm... this will require a lot more thought :-D [`D3.js`](https://d3js.org/) seems to be the solution here. It doesn't seem all that difficult to interface with. We could simply produce a static JSON representation of the ZK at build and feed it to D3, which is running in a "graph" page.
