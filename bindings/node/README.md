# @zero-edge/core

Node.js bindings for the [zero-edge](https://github.com/tonywied17/zero-edge)
device SDK core, built with [napi-rs](https://napi.rs).

The generated surface is intentionally thin. A hand-written, idiomatic layer is
added on top of it so JavaScript and TypeScript callers get a native-feeling API
while all behavior stays in the Rust core.

## Build

```
npm install
npm run build
npm test
```

`npm run build` compiles the Rust core into a native Node addon and emits
`index.js` and `index.d.ts`. These are generated artifacts and are not committed.
