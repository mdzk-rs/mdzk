To set the metadata of your mdzk, configure the build process and determine how the output will look, you use the `mdzk.toml`-file at the root of your mdzk. This [TOML](https://toml.io/) file has the following main tables:

- **mdzk**: Here you can set the metadata of your mdzk, as well as the source directory of your notes.
- **build**: Here you can specify the build process, for example the output directory, what preprocessors should be used, etc.
- **style**: Here the style of your mdzk can be configured. You can change titles, set CSS variables to be used in the HTML output and even inject custom CSS.

There are also more options supported.

<!-- TODO: A short guide and some simple examples -->
