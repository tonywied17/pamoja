# pamoja-lorawan

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

LoRaWAN 1.0.x MAC framing for the pamoja SDK.

LoRaWAN is how a low-power node reaches a network kilometres away over a license-free
radio, which is why it is the SDK's first-class answer for rural and remote reach. The
[`pamoja-lora`](https://docs.rs/pamoja-lora) crate gives the link budget, the exact
time a transmission spends on air; this crate gives the bytes that go in it: the
secured LoRaWAN frame.

A LoRaWAN frame is not just a payload with an address. The standard wraps every frame
in two cryptographic guarantees, because a long-range public-band link is wide open: a
message integrity code keyed to the network proves the frame is authentic and intact,
and the payload is encrypted to the application so only its owner can read it. This
crate builds and verifies exactly that, with no radio and no allocation:

- [`Session`] - an activated device's address and session keys. It [encodes](Session::encode_uplink)
  an uplink or downlink data frame, encrypting the payload and appending the MIC, and
  [decodes](Session::decode) one received, verifying the MIC before decrypting.
- [`Uplink`] and [`Downlink`] - the data frame to send, built up from the fields a
  sender sets (confirmed, adaptive data rate, acknowledgement, frame options).
- [`RxData`] - a decoded frame: its header fields and its recovered payload.
- [`Device`] - the root credentials for over-the-air activation: it builds the
  join-request a device broadcasts and turns the network's join-accept into a ready
  [`Session`], deriving the session keys the spec prescribes.

The cryptography is the LoRaWAN construction over AES-128: an AES-CMAC MIC and an
AES keystream for the payload, with the device address and frame counter folded into
both so a frame cannot be lifted out of its place in the stream. Driving the radio
arrives with the hardware-I/O layer; this is the secured-packet half ahead of it.

**Examples**

```
use pamoja_lorawan::{Session, Uplink};

// A node activated with a device address and its two session keys.
let session = Session::new(0x2601_1BDA, [0x2B; 16], [0x99; 16]);

// Encode a confirmed uplink reading; the payload is encrypted and the MIC appended.
let frame = session
    .encode_uplink(&Uplink::new(42, 1, b"temp=4.8").confirmed())
    .unwrap();

// The network, holding the same session, verifies and decrypts it.
let rx = session.decode(frame.as_bytes(), 42).unwrap();
assert!(rx.confirmed());
assert_eq!(rx.payload(), b"temp=4.8");
```

## Modules

- [crypto](crypto.md)
- [error](error.md)
- [frame](frame.md)
- [join](join.md)
- [session](session.md)

