//! Sessions: an ordered, replay-protected channel of authenticated-encrypted
//! messages between two devices that have agreed a key.

use crate::aead;
use crate::kex::{self, AgreementKey, AgreementPublicKey};
use crate::SessionError;

// The anti-replay window tracks the 64 counters below the highest accepted one, in a
// single machine word, exactly as the IPsec (RFC 4303) and DTLS (RFC 6347) sliding
// windows do.
const WINDOW: u64 = 64;

/// Which end of a session a device is.
///
/// Both ends derive the same key, but each tags its outgoing messages with a
/// different direction byte in the nonce, so the two directions never share a
/// nonce under the one key and a message a device sends can never be opened as one
/// it expected to receive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The device that opens the session. Its public key is ordered first when the
    /// key is derived, and it tags the messages it sends as the initiator direction.
    Initiator,
    /// The device that answers. Its public key is ordered second, and it tags the
    /// messages it sends as the responder direction.
    Responder,
}

/// The out-of-band header of a sealed message: the counter that orders it and the
/// tag that authenticates it.
///
/// Both values travel alongside the ciphertext to the peer. The peer needs the
/// counter to rebuild the nonce and to reject replays, and the tag to verify the
/// message was not altered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sealed {
    /// The monotonically increasing counter naming this message within the session.
    pub counter: u64,
    /// The 16-byte ChaCha20-Poly1305 tag over the ciphertext and its associated data.
    pub tag: [u8; 16],
}

/// A confidential, tamper-evident, replay-protected channel with one peer.
///
/// A session holds the agreed key, the counter for the messages this device sends,
/// and a sliding window of the counters it has accepted from the peer. Sealing a
/// message encrypts it and stamps it with the next counter; opening one verifies it
/// and rejects anything that fails authentication or repeats a counter.
///
/// A session is deliberately not `Clone`: two copies would reuse counters and so
/// reuse nonces, which breaks the AEAD's guarantees. Establish a fresh session
/// instead.
///
/// # Examples
///
/// ```
/// use pamoja_session::{AgreementKey, Role, Session};
///
/// // Each device is provisioned with its own seed and knows the other's public key.
/// let fridge = AgreementKey::from_seed(&[1u8; 32]);
/// let gateway = AgreementKey::from_seed(&[2u8; 32]);
///
/// // A fresh salt is agreed in the clear at the start of each session.
/// let salt = [9u8; 16];
/// let mut device = Session::establish(&fridge, &gateway.public(), &salt, Role::Initiator);
/// let mut peer = Session::establish(&gateway, &fridge.public(), &salt, Role::Responder);
///
/// // The device seals a reading; the ciphertext, counter, and tag go on the wire.
/// let mut message = *b"4.8C";
/// let sealed = device.seal(&mut message, b"fridge-1");
///
/// // The gateway opens it, recovering the reading and proving it is authentic.
/// peer.open(&sealed, &mut message, b"fridge-1").expect("authentic message");
/// assert_eq!(&message, b"4.8C");
/// ```
pub struct Session {
    key: [u8; 32],
    nonce_prefix: [u8; 3],
    send_dir: u8,
    recv_dir: u8,
    send_counter: u64,
    recv_highest: u64,
    recv_window: u64,
}

impl Session {
    /// Establishes a session with a peer from this device's agreement key and the
    /// peer's authenticated public key.
    ///
    /// Both devices call this with the same `salt` and opposite [`Role`]s and arrive
    /// at the same key. The salt is a fresh per-session value the two sides exchange
    /// in the clear before sealing anything; reusing a salt with the same pair of
    /// keys reuses the session key, so it must change each session (a counter kept in
    /// power-loss-safe storage, or a nonce from a handshake, both work).
    ///
    /// # Arguments
    ///
    /// * `local` - this device's key-agreement secret.
    /// * `peer` - the peer's public key, already authenticated by pinning or signature.
    /// * `salt` - the fresh per-session salt both sides share.
    /// * `role` - whether this device is the [`Role::Initiator`] or [`Role::Responder`].
    ///
    /// # Returns
    ///
    /// A session ready to seal and open messages with the peer.
    pub fn establish(
        local: &AgreementKey,
        peer: &AgreementPublicKey,
        salt: &[u8],
        role: Role,
    ) -> Self {
        let shared = local.shared_secret(peer);
        let local_public = local.public().to_bytes();
        let peer_public = peer.to_bytes();
        let (initiator, responder) = match role {
            Role::Initiator => (local_public, peer_public),
            Role::Responder => (peer_public, local_public),
        };

        let okm = kex::derive(&shared, salt, &initiator, &responder);
        let mut key = [0u8; 32];
        key.copy_from_slice(&okm[..32]);
        let mut nonce_prefix = [0u8; 3];
        nonce_prefix.copy_from_slice(&okm[32..]);

        let (send_dir, recv_dir) = match role {
            Role::Initiator => (0, 1),
            Role::Responder => (1, 0),
        };

        Self {
            key,
            nonce_prefix,
            send_dir,
            recv_dir,
            send_counter: 0,
            recv_highest: 0,
            recv_window: 0,
        }
    }

    /// Seals a message for the peer, encrypting `buf` in place and stamping it with
    /// the next counter.
    ///
    /// The associated data `aad` is authenticated but not encrypted, so it is
    /// readable on the wire yet cannot be altered: a device identifier or a routing
    /// header belongs here. After this returns, `buf` holds the ciphertext and the
    /// returned [`Sealed`] holds the counter and tag to send with it.
    ///
    /// # Arguments
    ///
    /// * `buf` - the plaintext, replaced in place by the ciphertext of equal length.
    /// * `aad` - associated data to authenticate alongside the message.
    ///
    /// # Returns
    ///
    /// The [`Sealed`] header (counter and tag) for this message.
    pub fn seal(&mut self, buf: &mut [u8], aad: &[u8]) -> Sealed {
        let counter = self.send_counter;
        let nonce = nonce(&self.nonce_prefix, self.send_dir, counter);
        let tag = aead::seal(&self.key, &nonce, aad, buf);
        // A session must be re-established long before 2^64 messages; this never wraps
        // in any real deployment.
        self.send_counter += 1;
        Sealed { counter, tag }
    }

    /// Opens a message from the peer, verifying it and decrypting `buf` in place.
    ///
    /// The message is rejected if its counter has already been seen or is older than
    /// the replay window still tracks, and if its tag does not authenticate. On any
    /// rejection `buf` is left zeroed, so a failed open never yields readable bytes.
    /// The replay window only advances on a message that authenticates, so a forged
    /// counter cannot push genuine messages out of the window.
    ///
    /// # Arguments
    ///
    /// * `sealed` - the counter and tag that arrived with the ciphertext.
    /// * `buf` - the ciphertext, replaced in place by the plaintext on success.
    /// * `aad` - the same associated data the sender authenticated.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message is authentic and fresh, with `buf` now the plaintext.
    ///
    /// # Errors
    ///
    /// Returns [`SessionError::Replayed`] if the counter repeats or is too old, or
    /// [`SessionError::Inauthentic`] if the message fails authentication.
    pub fn open(
        &mut self,
        sealed: &Sealed,
        buf: &mut [u8],
        aad: &[u8],
    ) -> Result<(), SessionError> {
        if !self.replay_ok(sealed.counter) {
            return Err(SessionError::Replayed);
        }
        let nonce = nonce(&self.nonce_prefix, self.recv_dir, sealed.counter);
        aead::open(&self.key, &nonce, aad, buf, &sealed.tag)?;
        self.commit(sealed.counter);
        Ok(())
    }

    // Whether `counter` could still be accepted: it is newer than the highest seen,
    // or it falls inside the window and has not been seen there yet.
    fn replay_ok(&self, counter: u64) -> bool {
        if counter > self.recv_highest {
            return true;
        }
        let behind = self.recv_highest - counter;
        if behind >= WINDOW {
            return false;
        }
        (self.recv_window >> behind) & 1 == 0
    }

    // Records `counter` as accepted, sliding the window forward if it is a new high.
    fn commit(&mut self, counter: u64) {
        if counter > self.recv_highest {
            let shift = counter - self.recv_highest;
            self.recv_window = if shift >= WINDOW {
                1
            } else {
                (self.recv_window << shift) | 1
            };
            self.recv_highest = counter;
        } else {
            let behind = self.recv_highest - counter;
            self.recv_window |= 1 << behind;
        }
    }
}

// Builds the 12-byte ChaCha20-Poly1305 nonce: a direction byte, the per-session
// prefix, and the big-endian counter. The direction byte separates the two
// directions under the shared key, and the counter makes every nonce in a direction
// unique, which is the discipline the AEAD requires.
fn nonce(prefix: &[u8; 3], direction: u8, counter: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[0] = direction;
    nonce[1..4].copy_from_slice(prefix);
    nonce[4..].copy_from_slice(&counter.to_be_bytes());
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Session, Session) {
        let initiator = AgreementKey::from_seed(&[1u8; 32]);
        let responder = AgreementKey::from_seed(&[2u8; 32]);
        let salt = [3u8; 16];
        let a = Session::establish(&initiator, &responder.public(), &salt, Role::Initiator);
        let b = Session::establish(&responder, &initiator.public(), &salt, Role::Responder);
        (a, b)
    }

    #[test]
    fn a_sealed_message_opens_on_the_peer() {
        let (mut a, mut b) = pair();
        let mut buf = *b"hello";
        let sealed = a.seal(&mut buf, b"meta");
        b.open(&sealed, &mut buf, b"meta").expect("authentic");
        assert_eq!(&buf, b"hello");
    }

    #[test]
    fn the_two_sides_derive_the_same_key() {
        // If the keys or directions disagreed, this cross-direction exchange would
        // fail to authenticate.
        let (mut a, mut b) = pair();
        let mut up = *b"up";
        let sealed_up = a.seal(&mut up, b"");
        b.open(&sealed_up, &mut up, b"").expect("a to b");
        let mut down = *b"down";
        let sealed_down = b.seal(&mut down, b"");
        a.open(&sealed_down, &mut down, b"").expect("b to a");
    }

    #[test]
    fn a_replayed_message_is_rejected() {
        let (mut a, mut b) = pair();
        let mut buf = *b"once";
        let sealed = a.seal(&mut buf, b"");
        let mut first = buf;
        b.open(&sealed, &mut first, b"").expect("first delivery");
        let mut again = buf;
        assert_eq!(
            b.open(&sealed, &mut again, b""),
            Err(SessionError::Replayed)
        );
    }

    #[test]
    fn out_of_order_within_the_window_is_accepted_once_each() {
        let (mut a, mut b) = pair();
        let mut payloads = [*b"00", *b"01", *b"02", *b"03"];
        let sealed: [Sealed; 4] = core::array::from_fn(|i| a.seal(&mut payloads[i], b""));
        // Deliver newest first, then the older ones: all fresh, all accepted.
        for i in [3, 1, 2, 0] {
            let mut buf = payloads[i];
            b.open(&sealed[i], &mut buf, b"")
                .expect("fresh within window");
        }
        // Re-delivering any of them now repeats a counter already in the window.
        let mut buf = payloads[2];
        assert_eq!(
            b.open(&sealed[2], &mut buf, b""),
            Err(SessionError::Replayed)
        );
    }

    #[test]
    fn a_counter_older_than_the_window_is_rejected() {
        let (mut a, mut b) = pair();
        // Advance the receiver past a full window with a high counter.
        a.send_counter = 100;
        let mut new = *b"new";
        let sealed_new = a.seal(&mut new, b"");
        b.open(&sealed_new, &mut new, b"")
            .expect("new high counter");
        // A message at counter 0 is now far below the window's reach.
        a.send_counter = 0;
        let mut old = *b"old";
        let sealed_old = a.seal(&mut old, b"");
        assert_eq!(
            b.open(&sealed_old, &mut old, b""),
            Err(SessionError::Replayed)
        );
    }

    #[test]
    fn a_forged_tag_does_not_advance_the_window() {
        let (mut a, mut b) = pair();
        // A forgery at a high counter must not push the window forward.
        let forged = Sealed {
            counter: 50,
            tag: [0u8; 16],
        };
        let mut junk = *b"junk";
        assert_eq!(
            b.open(&forged, &mut junk, b""),
            Err(SessionError::Inauthentic)
        );
        // The genuine first message still opens, proving the window did not move.
        let mut buf = *b"first";
        let sealed = a.seal(&mut buf, b"");
        b.open(&sealed, &mut buf, b"")
            .expect("window was not advanced by the forgery");
    }

    #[test]
    fn a_different_salt_yields_an_incompatible_session() {
        let initiator = AgreementKey::from_seed(&[1u8; 32]);
        let responder = AgreementKey::from_seed(&[2u8; 32]);
        let mut a =
            Session::establish(&initiator, &responder.public(), &[3u8; 16], Role::Initiator);
        let mut b =
            Session::establish(&responder, &initiator.public(), &[4u8; 16], Role::Responder);
        let mut buf = *b"hello";
        let sealed = a.seal(&mut buf, b"");
        assert_eq!(
            b.open(&sealed, &mut buf, b""),
            Err(SessionError::Inauthentic)
        );
    }
}
