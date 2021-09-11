# Changelog

## 0.3.8 (Unreleased)

### New features

- You can now choose a renderer with the `--renderer` or `-r` flag when running `build` or `serve`. The choices are between `mdbook`, `markdown` or `mdzk`, the latter being our own work in progress custom renderer. This feature is mostly for debugging purposes.

### Enhancements

- The mdzk renderer now copies over KaTeX locally in the output, to ensure speedy math rendering.
- The mdzk renderer now uses the Inter font.

## 0.3.7 (2021-09-05)

0.3.7 moves mdzk to a monorepo structure. No other changes.

## 0.3.6 (2021-09-03)

### Enhancements

- The binary's size is now a lot smaller. (f5e6ca4b4bc98e98a3f28a5859665a332ff10262)
- Sorting now works with non-ASCII characters. (16bb74d62314c4bdaa75809b64caae3e9b2ec7dc)

## 0.3.5 (2021-08-31)

### Enhancements

- Summary is now sorted by directories first. (78f98d040fe4674c34571d115d8e7de10c14eb97)

### Bug fixed

- Added `*.hbs` files to Cargo.toml. (3b4bcd98374b4463143fd81cf621b97c198bafae)

## 0.3.4 (2021-08-27)

### Enhancements

- Moved to `lazy_regex` for lazy evaluation of regexes instead of doing this manually with `lazy_static`. This hopefully brings some minor speed improvements.

### Bug fixes

- `'` and `"` threw errors when compiling, because these characters were escaped in their filename, meaning mdzk could not find the source.
- The URLs produced by our preprocessors did not escape special characters properly. This is now fixed, meaning URLs should all work properly again. Unsupported characters are documented in the [README](https://github.com/mdzk-rs/mdzk/blob/main/README.md), but repeated here: `=`, `\``, `^`, `#`, `|`, `:`, `/`, `[` and `]`. `$` is supported, but not recommended, as it might intefere with the KaTeX math delimiters.

## 0.3.3 (2021-08-24)

### New features

- New front matter field: Use the `date` (string) field to add a date to the note. The datestring must comply with [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601), with seconds and timezone being optional. If no timezone is specified, the local timezone will be used, though the timezone information is not utilized at the moment.

### Enhancements

- You are now able to use [TOML](https://toml.io/en/) in the front matter, if that suits your fancy.
- The [`IndexPreprocessor`](https://github.com/rust-lang/mdBook/blob/master/src/preprocess/index.rs) bundled with mdBook is improved for mdzk and now replaces all links to `README.md` into `index.md`. This means you can use the Markdown convention of naming the main page README without having broken links/backlinks.


## 0.3.2 (2021-08-13)

### Bug fixes

- Once again fixed a hot-reloading loop caused by tracking changes on the summmary file.


## 0.3.1 (2021-08-13)

### Enhancements

- Prettier log messages. There should be a lot less verbosity when running mdzk now.


## 0.3.0 (2021-08-12)

### Breaking changes

- mdzk now has a set of defined global variables for the directory tree:
    
    - Source directory: `notes`
    - Build directory: `html`
    - (!) Config file: `mdzk.toml`
    - (!) Summary file: `.mdzk_summary.md` (located within the source directory)

    You will (sadly) have to change the files marked with (!) manually for the new version of mdzk to work.
    
    The advantage of hiding the summary file is that Obsidian users can enjoy their graph view without it being ruined by one monolithic `SUMMARY.md` node.

### Enhancements

- `mdzk init` produces a directory tree completely according to the above specification. It does not pollute this tree with any default files like mdBook's `chapter_1.md`.
- We (hopefully) now use better HTML escaping, which means characters like `"`, `'` and `&` should be safer to use.

### Bug fixes

- Both wikilinks and backlinks now use relative paths, which means they should be completely functional regardless of how nested your notes are or how the vault is published.


## 0.2.4 (2021-08-10)

Sporting the newest additions to `mdbook-wikilink` and `mdbook-backlinks`.

### New features

- Disable default processors with the `disable_default_processors` option.


## 0.2.3 (2021-08-09)

### Bug fixes

- `mdzk serve` did not load the correct config, nor update the `SUMMARY.md` file on file changes. This is now fixed.


## 0.2.2 (2021-08-06)

### New features

- added the flags `port` and `bind` to the `serve` subcommand (<https://github.com/mdzk-rs/mdzk/commit/033c071c9cf2a4855707a3ec0db0a612a82601ab>)
