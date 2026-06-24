# pamoja-session::aead

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

ChaCha20-Poly1305 authenticated encryption (RFC 8439), the AEAD that protects
every message a session sends.

ChaCha20-Poly1305 is chosen over AES-GCM because the cheap hardware this SDK
targets rarely has AES acceleration, and ChaCha20 is fast and constant-time in
plain software. The construction is used through the vetted RustCrypto
implementation; this module only wraps it in the in-place, detached-tag shape a
session needs, and the tests pin it to the worked example in RFC 8439 section
2.8.2 so a wrong key, nonce, or associated-data wiring is caught.

