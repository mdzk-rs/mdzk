# Changelog

## 0.5.0 (Unreleased)

### New features

- [#55](https://github.com/mdzk-rs/mdzk/pull/55): You can now turn off each preprocessor feature of mdzk separately. This is done with the following configuration values:

    ```toml
    [build]
    # ...
    front-matter = false
    math = false
    readme = false
    math = false
    ```

    More about these features and what they do, can be found in the [documentation](https://mdzk.app/docs).

### Enhancements

- [#54](https://github.com/mdzk-rs/mdzk/issues/54): Our separate preprocessor crates are now moved to within the mdzk crate. This allows us to avoid iterating over the vault multiple times, which drastically increases the efficiency.
- [#45](https://github.com/mdzk-rs/mdzk/issues/45): New custom made user interface, for a friendlier look. Expect more informative messages from this update and on.
- Enabled pulldown-cmark's SIMD accelerated scanners.

### Bug fixes

- [#41](https://github.com/mdzk-rs/mdzk/pull/41): mdzk will not panic on invalid TOML anymore, but rather print an error message coupled with the configuration content and a description of what part of it is invalid.
- [#40](https://github.com/mdzk-rs/mdzk/pull/40): mdzk now checks whether you updated your `mdzk.toml` correctly or not. If a `[book]` section is found in your `mdzk.toml` file, a warning will appear that prompts the user to change it to `[mdzk]`.

## 0.4.3 (2020-11-01)

### New features

- New `generate-summary` key at the `[mdzk]` section in `mdzk.toml` that controls whether mdzk will generate your summary file automatically or not. The default value is `true`.
- New `preprocessors` field under the `[build]` section in `mdzk.toml`. This lets you define [mdBook preprocessors](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html) to be run after the default preprocessors. Order is preserved.
- New `draft` option in the front matter. Setting this to `true` will skip rendering for that note.

    **NOTE**: Beware that the note will still show up in the index, as that is how mdBook handles draft chapters. We will probably change this behaviour as soon as we release our custom renderer. If you want to hide a note completely from your vault, use the `ignore` field in `mdzk.toml`.

### Enhancements

- Removed several dangerous `unwrap`s, to make the codebase more robust.
- The Nix build now supports Apple Silicon as well.
- Dependencies have been cleaned up, leading to a slightly smaller binary size.
- New version of mdbook-wiklinks (0.4.0), that replaces regexes in favor of a combination of [pulldown-cmark's](https://docs.rs/pulldown-cmark) iteration and a custom [Pest](https://pest.rs/) parser. This should decrease build times drastically.
- Math delimiters are now converted into spans/divs on build. This will hopefully make KaTeX support more robust, and ignore math rendering within code and links.

### Bug fixes

- [#38](https://github.com/mdzk-rs/mdzk/issues): User-defined preprocessors (through the mdBook-style `preprocessor.<name>` pattern in the config file) would sometimes interfere with mdzk's default preprocessors. This fix converts all of these preprocessor-statements into the `preprocessors` field mentioned above, which mdzk can handle the order of.
    
    **NOTE**: To simplify compatibility between mdzk and mdbook, we assume the preprocessor has a command on the form `mdbook-<name>`. Other preprocessors are simply not supported yet. Fields are ignored, so you can only turn preprocessors on or off.

- [#24](https://github.com/mdzk-rs/mdzk/issues/24): `mdzk.backlinks-header` in the config had no defuault value, which made mdzk panic when it was not set. This is now fixed.

## 0.4.2 (2020-09-24)

This is a quick update that should make compiling mdzk easier (and even possible) for Windows users.

We have also experienced some trouble with Windows Defender flagging pre-compiled mdzk binaries as a trojan. **This is a false positive.** A report is submitted to Microsoft, and we have removed all previously released Windows binaries. We suspect the warning is related to the amount of embedded JavaScript that is contained in mdzk. Hopefully, you will not get any warnings from this version and onwards. If you do, please leave a comment with a screenshot of the warning in [this issue](https://github.com/mdzk-rs/mdzk/issues/22).

### New features

- When naming a file the same name as it's parent directory, this file will work as the content of that directory. Namely, the directory will itself function as a note.

### Bug fixes

- Our dependency on [`mdbook-katex`](https://github.com/lzanini/mdbook-katex), and subsequently QuickJS, did not work well with most Windows computers. To circumvent this, KaTeX is now loaded from a CDN and handles the rendering client-side. This shouldn't lead to any significant increases in load time, but it is not ideal for offline usage. We are working on embedding KaTeX locally in the generated output, which hopefully will be included soon.

## 0.4.1 (2020-09-23)

### Enhancements

- The mdzk renderer now converts `.md` into `.html` in links/images during rendering.

### Bug fixes

- Fixed `init` command. It did not produce a correctly formatted config file, and a subsequent build failed.

## 0.4.0 (2021-09-20)

### Breaking changes

- The `[book]` table header in `mdzk.toml` is now changed to `[mdzk]`. You will need to change this in your own configuration, or else the values will be ignored.
- The `disable-default-processors` configuration option is now placed under the `[build]` table.

### New features

- An `ignore` field is added under the `[mdzk]` table. This is a list of gitignore-style patterns that specify files or directories which mdzk should ignore when building.
- The header above the backlinks can now also be configured under the `[mdzk]` table, with the `backlinks-header` field. Both this and the previous `preprocessor.backlinks.header` field are supported, but you are recommended to use the former, in case we might remove the latter.
- You can now choose a renderer with the `--renderer` or `-r` flag when running `build` or `serve`. The choices are between `mdbook`, `markdown` or `mdzk`, the latter being our own work in progress custom renderer. This feature is mostly for debugging purposes.

### Enhancements

- We now use our own configuration struct. This will allow us to more easily expand mdzk with more options and configuration.
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
