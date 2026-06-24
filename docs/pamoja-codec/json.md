# pamoja-codec::json

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A human-readable JSON codec built on [`serde_json`].

## struct `JsonCodec`

A codec that serializes values to and from JSON.

JSON trades the compactness of CBOR for human readability, which makes it a
good fit for debugging, configuration, and interop with services that speak
JSON. The codec works for any type that implements [`serde::Serialize`] and
[`serde::de::DeserializeOwned`].

**Examples**

```
use pamoja_codec::{Codec, JsonCodec};

let codec = JsonCodec;
let encoded = codec.encode(&vec![1, 2, 3]).unwrap();
assert_eq!(encoded, b"[1,2,3]");
let decoded: Vec<i32> = codec.decode(&encoded).unwrap();
assert_eq!(decoded, vec![1, 2, 3]);
```

