# Changelog


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
