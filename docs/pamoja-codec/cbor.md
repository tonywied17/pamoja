# pamoja-codec::cbor

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

A compact CBOR codec built on [`ciborium`].

## struct `CborCodec`

A codec that serializes values to and from CBOR.

CBOR is a compact, self-describing binary format, which makes it the default
choice for constrained devices and metered radio links where every byte costs
power or money. The codec works for any type that implements [`serde::Serialize`]
and [`serde::de::DeserializeOwned`].

**Examples**

```
use pamoja_codec::{CborCodec, Codec};

let codec = CborCodec;
let value = (1u8, "ok".to_owned());
let encoded = codec.encode(&value).unwrap();
let decoded: (u8, String) = codec.decode(&encoded).unwrap();
assert_eq!(decoded, value);
```

