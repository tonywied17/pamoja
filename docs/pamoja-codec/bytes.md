# pamoja-codec::bytes

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A codec that carries raw bytes unchanged.

## struct `BytesCodec`

A no-op codec for payloads that are already byte buffers.

Encoding clones the buffer and decoding copies the input, so values pass
through unchanged. This is the "raw framing" option for payloads that carry
their own format, such as an image chunk or a pre-encoded frame.

**Examples**

```
use pamoja_codec::{BytesCodec, Codec};

let codec = BytesCodec;
let payload = vec![0xde, 0xad, 0xbe, 0xef];
let encoded = codec.encode(&payload).unwrap();
assert_eq!(codec.decode(&encoded).unwrap(), payload);
```

