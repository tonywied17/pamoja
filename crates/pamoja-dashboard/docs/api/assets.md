# assets

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The static page assets, served either embedded or live from disk.

In production the page is baked into the firmware with `include_bytes!`, so there
is no filesystem dependency and the dashboard is part of the image. In development
the same directory is read from disk on every request, so editing the app and
reloading shows the change with no recompile. The dashboard is a multi-file
ES-module zQuery app, so both modes resolve any nested path under the web root.

## enum `Assets`

Where the page assets come from.

- `Embedded` - Baked into the binary at compile time: the production path.
- `Dir` - Read from a directory on each request: the hot-reloading development path.

### `Assets::get`

Resolves a request path to a file's MIME type and bytes.

The request path `"/"` resolves to the page shell. In [`Assets::Dir`] mode any
file under the directory is served (typed by extension), read fresh from disk so
edits show up on reload; a path that escapes the directory resolves to `None`.

**Arguments**

* `path` - the request path, such as `"/app/app.js"`.

**Returns**

The MIME type and the file's bytes, or `None` if no asset matches.

```rust
fn get(&self, path: &str) -> Option <(&'static str, Vec <u8>)>
```

