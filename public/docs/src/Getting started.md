---
title: Getting started
---

In mdzk, a Zettelkasten is simply a directory containing two things: Some Markdown-files and a TOML configuration file. To simplify things, we'll make a new noun and refer to this directory (and it's output) as "your *mdzk*".

This getting started guide will go over setting up your own mdzk, adding notes to it and generating a static webpage from them.

### Initializing your mdzk

First, navigate to the directory you want to use as your mdzk and run

    $ mdzk init

Optionally, you can have mdzk create a new directory for you, by running `mdzk init <path>`. You will notice that the following has been generated:

- A TOML configuration file at the root of your mdzk, called `mdzk.toml`.

    > 📖 *This file is used to specify metadata about your mdzk, build instructions and much more. You can read more about configuring [[Configuration|here]].*

- A directory called `notes`.

    > 📖 *This is where you put all your notes (duh). If you want, you can change which path (relative to the root) holds your source notes by changing the `src` value in `mdzk.toml`.*

- A `.gitignore` file.

There are no notes in your mdzk yet, so nothing exciting will happen. This is your time to shine - let's write some content!

<!-- TODO: Adding existing notes -->

### Writing notes

Now is the time to open the `notes` folder in your favorite text editor. Start by creating some Markdown-files (`.md`) and write whatever your heart desires. Create some subdirectories if you want, and put notes in there. *Go buck wild!* If you have a collection of Markdown-notes already, this is the time to copy them over.

All notes follow an extended variant of the [CommonMark](https://commonmark.org/) specification. An excellent overview of all it's features is available [here](https://commonmark.org/help/). If you are unfamiliar with Markdown, we recommend you follow [this quick interactive tutorial](https://commonmark.org/help/tutorial/) to get comfortable with the markup language.

Among the main extensions mdzk provides to CommonMark are internal links. Internal links are made using the *wikilink* syntax, e.g.: `[[Name of destination note]]`. Say you have a note called `White bellbird's mating call.md`. If you want to make a link to this note from another, you can write

```markdown
I'm trying to pay attention and write notes from this lecture, but I'm too bored. I'd rather listen to a [[White bellbird's mating call]] than suffer another second.
```

This will create a link in the text and insert a backlink at the end of `White bellbird's mating call.md`. Note that the `.md` extension is excluded in internal links. More about mdzk's markup can be read [[Markup|here]].

> 📖 *For a fully specialized experience, our text editor recommendation goes to [Obsidian](https://obsidian.md). mdzk is targeted specifically at it's syntax and is inspired by many of it's conventions. However, any text editor will suffice. I personally love writing my notes in [Neovim](https://neovim.io/), and there are many extensions that add wikilink support to e.g. [Visual Studio Code](https://code.visualstudio.com/). Use whatever suits you best.*

### Building a static site

After you've added some notes, simply run

    $ mdzk build

to generate a static website. By default, everything is generated in the `html` directory at the root of your mdzk. This site is viewable locally by opening any of the generated HTML files, or it can be hosted on any static publishing platform - like [GitHub Pages](https://pages.github.com/).

mdzk also provides a webserver, so you can view your notes with live updating as you change them. To serve your mdzk locally, run:

    $ mdzk serve

Your notes are now available at <http://localhost:3000>.
