//! X25519 key agreement (RFC 7748) and HKDF-SHA256 (RFC 5869): how two devices that
//! already hold each other's public key arrive at the same session key without ever
//! sending it.
//!
//! The raw X25519 shared secret is never used as a key directly. It is run through
//! HKDF-SHA256, salted with a fresh per-session value and bound to both public keys,
//! so each session gets an independent key and the key is tied to the specific pair
//! of devices. The tests pin both primitives to their RFC reference vectors.

use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

// The label mixed into every derivation, so a pamoja session key can never collide
// with a key some other protocol derives from the same shared secret.
const CONTEXT: &[u8] = b"pamoja-session v1";

// The derived output: 32 bytes of session key followed by a 3-byte nonce prefix.
pub(crate) const OKM_LEN: usize = 35;

/// A device's long-term key-agreement secret.
///
/// This is the private half a device uses to agree a session key with a peer. It is
/// built from a 32-byte seed the device is provisioned with and keeps in secure
/// storage, so the same agreement key is recreated deterministically across reboots.
///
/// It is separate from the device's ed25519 signing identity in `pamoja-security`.
/// Key agreement gives confidentiality; it does not by itself prove who the peer is.
/// A deployment authenticates the peer by pinning its [`public`](AgreementKey::public)
/// value, or by signing that value with the peer's `pamoja-security` identity, the
/// same way it already pins a signing identity. Without that pinning the channel is
/// private but unauthenticated and a man in the middle is possible.
///
/// # Examples
///
/// ```
/// use pamoja_session::AgreementKey;
///
/// let device = AgreementKey::from_seed(&[7u8; 32]);
/// let public = device.public();
/// // `public.to_bytes()` is what a peer pins or has signed to trust this device.
/// assert_eq!(public.to_bytes().len(), 32);
/// ```
pub struct AgreementKey {
    secret: StaticSecret,
}

impl AgreementKey {
    /// Builds a key-agreement secret from a 32-byte seed.
    ///
    /// # Arguments
    ///
    /// * `seed` - the 32 secret bytes the key is derived from.
    ///
    /// # Returns
    ///
    /// The agreement key.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            secret: StaticSecret::from(*seed),
        }
    }

    /// Returns the public key a peer needs to agree a session with this device.
    ///
    /// # Returns
    ///
    /// The matching [`AgreementPublicKey`], safe to share once it is authenticated.
    pub fn public(&self) -> AgreementPublicKey {
        AgreementPublicKey {
            inner: PublicKey::from(&self.secret),
        }
    }

    // Computes the raw X25519 shared secret with a peer. Callers feed this into
    // `derive`; it is never used as a key on its own.
    pub(crate) fn shared_secret(&self, peer: &AgreementPublicKey) -> [u8; 32] {
        self.secret.diffie_hellman(&peer.inner).to_bytes()
    }
}

/// The public half of a device's key-agreement key.
///
/// A device holds the authenticated public keys of the peers it will talk to and
/// uses them to agree a session key. It is 32 bytes on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgreementPublicKey {
    inner: PublicKey,
}

impl AgreementPublicKey {
    /// Reconstructs a public key from its 32-byte form.
    ///
    /// # Arguments
    ///
    /// * `bytes` - the 32-byte encoded public key.
    ///
    /// # Returns
    ///
    /// The public key. Every 32-byte value is a syntactically valid X25519 public
    /// key, so this cannot fail; authenticating that the key belongs to the expected
    /// device is the caller's responsibility.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            inner: PublicKey::from(*bytes),
        }
    }

    /// Returns the 32-byte wire form of this public key.
    ///
    /// # Returns
    ///
    /// The public key encoded as 32 bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.inner.to_bytes()
    }
}

// Derives the session key material from a shared secret, salted per session and bound
// to both public keys. Returns 32 key bytes plus a 3-byte nonce prefix.
pub(crate) fn derive(
    shared: &[u8; 32],
    salt: &[u8],
    initiator: &[u8; 32],
    responder: &[u8; 32],
) -> [u8; OKM_LEN] {
    const INFO_LEN: usize = CONTEXT.len() + 64;
    let mut info = [0u8; INFO_LEN];
    info[..CONTEXT.len()].copy_from_slice(CONTEXT);
    info[CONTEXT.len()..CONTEXT.len() + 32].copy_from_slice(initiator);
    info[CONTEXT.len() + 32..].copy_from_slice(responder);

    let mut okm = [0u8; OKM_LEN];
    hkdf_sha256(salt, shared, &info, &mut okm);
    okm
}

// HKDF-SHA256 (RFC 5869): extract a pseudorandom key from `salt` and `ikm`, then
// expand it under `info` into `out`. Kept as its own function so it can be pinned to
// the RFC's published test vector.
fn hkdf_sha256(salt: &[u8], ikm: &[u8], info: &[u8], out: &mut [u8]) {
    Hkdf::<Sha256>::new(Some(salt), ikm)
        .expand(info, out)
        .expect("output length is within HKDF-SHA256's 255 * 32-byte limit");
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 7748 section 6.1: Alice and Bob, with the public keys and shared secret the
    // example computes.
    const ALICE_SEED: [u8; 32] = [
        0x77, 0x07, 0x6d, 0x0a, 0x73, 0x18, 0xa5, 0x7d, 0x3c, 0x16, 0xc1, 0x72, 0x51, 0xb2, 0x66,
        0x45, 0xdf, 0x4c, 0x2f, 0x87, 0xeb, 0xc0, 0x99, 0x2a, 0xb1, 0x77, 0xfb, 0xa5, 0x1d, 0xb9,
        0x2c, 0x2a,
    ];
    const ALICE_PUBLIC: [u8; 32] = [
        0x85, 0x20, 0xf0, 0x09, 0x89, 0x30, 0xa7, 0x54, 0x74, 0x8b, 0x7d, 0xdc, 0xb4, 0x3e, 0xf7,
        0x5a, 0x0d, 0xbf, 0x3a, 0x0d, 0x26, 0x38, 0x1a, 0xf4, 0xeb, 0xa4, 0xa9, 0x8e, 0xaa, 0x9b,
        0x4e, 0x6a,
    ];
    const BOB_SEED: [u8; 32] = [
        0x5d, 0xab, 0x08, 0x7e, 0x62, 0x4a, 0x8a, 0x4b, 0x79, 0xe1, 0x7f, 0x8b, 0x83, 0x80, 0x0e,
        0xe6, 0x6f, 0x3b, 0xb1, 0x29, 0x26, 0x18, 0xb6, 0xfd, 0x1c, 0x2f, 0x8b, 0x27, 0xff, 0x88,
        0xe0, 0xeb,
    ];
    const BOB_PUBLIC: [u8; 32] = [
        0xde, 0x9e, 0xdb, 0x7d, 0x7b, 0x7d, 0xc1, 0xb4, 0xd3, 0x5b, 0x61, 0xc2, 0xec, 0xe4, 0x35,
        0x37, 0x3f, 0x83, 0x43, 0xc8, 0x5b, 0x78, 0x67, 0x4d, 0xad, 0xfc, 0x7e, 0x14, 0x6f, 0x88,
        0x2b, 0x4f,
    ];
    const SHARED: [u8; 32] = [
        0x4a, 0x5d, 0x9d, 0x5b, 0xa4, 0xce, 0x2d, 0xe1, 0x72, 0x8e, 0x3b, 0xf4, 0x80, 0x35, 0x0f,
        0x25, 0xe0, 0x7e, 0x21, 0xc9, 0x47, 0xd1, 0x9e, 0x33, 0x76, 0xf0, 0x9b, 0x3c, 0x1e, 0x16,
        0x17, 0x42,
    ];

    #[test]
    fn public_keys_match_the_rfc_7748_vector() {
        assert_eq!(
            AgreementKey::from_seed(&ALICE_SEED).public().to_bytes(),
            ALICE_PUBLIC
        );
        assert_eq!(
            AgreementKey::from_seed(&BOB_SEED).public().to_bytes(),
            BOB_PUBLIC
        );
    }

    #[test]
    fn both_sides_agree_the_rfc_7748_shared_secret() {
        let alice = AgreementKey::from_seed(&ALICE_SEED);
        let bob = AgreementKey::from_seed(&BOB_SEED);
        assert_eq!(alice.shared_secret(&bob.public()), SHARED);
        assert_eq!(bob.shared_secret(&alice.public()), SHARED);
    }

    #[test]
    fn a_public_key_round_trips_through_bytes() {
        let public = AgreementKey::from_seed(&ALICE_SEED).public();
        assert_eq!(AgreementPublicKey::from_bytes(&public.to_bytes()), public);
    }

    #[test]
    fn hkdf_matches_the_rfc_5869_basic_vector() {
        // RFC 5869 Appendix A.1: the SHA-256 basic test case.
        let ikm = [0x0bu8; 22];
        let salt: [u8; 13] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
        ];
        let info: [u8; 10] = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
        let expected: [u8; 42] = [
            0x3c, 0xb2, 0x5f, 0x25, 0xfa, 0xac, 0xd5, 0x7a, 0x90, 0x43, 0x4f, 0x64, 0xd0, 0x36,
            0x2f, 0x2a, 0x2d, 0x2d, 0x0a, 0x90, 0xcf, 0x1a, 0x5a, 0x4c, 0x5d, 0xb0, 0x2d, 0x56,
            0xec, 0xc4, 0xc5, 0xbf, 0x34, 0x00, 0x72, 0x08, 0xd5, 0xb8, 0x87, 0x18, 0x58, 0x65,
        ];
        let mut okm = [0u8; 42];
        hkdf_sha256(&salt, &ikm, &info, &mut okm);
        assert_eq!(okm, expected);
    }

    #[test]
    fn derive_changes_with_the_salt() {
        let first = derive(&SHARED, b"salt-one", &ALICE_PUBLIC, &BOB_PUBLIC);
        let second = derive(&SHARED, b"salt-two", &ALICE_PUBLIC, &BOB_PUBLIC);
        assert_ne!(first, second);
    }
}
