//! J1939: the meaning packed into a 29-bit CAN identifier.
//!
//! J1939 is the protocol trucks, tractors, marine engines, and generators speak over CAN.
//! It carries most of its addressing in the extended identifier itself: a priority, a
//! parameter group number that names what the message is, and the source (and sometimes
//! destination) address. This decodes that identifier and composes one.

use crate::id::CanId;

// PDU formats below this value carry a destination address in the PS field (PDU1);
// formats at or above it are broadcast, with PS a group extension (PDU2).
const PDU1_LIMIT: u8 = 240;

/// The fields a J1939 message packs into its 29-bit identifier.
///
/// # Examples
///
/// ```
/// use pamoja_can::{CanId, J1939Id};
///
/// // The standard engine-speed broadcast, identifier 0x0CF00400.
/// let message = J1939Id::from_id(CanId::extended(0x0CF0_0400)).unwrap();
/// assert_eq!(message.priority(), 3);
/// assert_eq!(message.pgn(), 61_444);
/// assert_eq!(message.source(), 0x00);
/// assert!(message.is_broadcast());
/// assert_eq!(message.destination(), None);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct J1939Id {
    priority: u8,
    pgn: u32,
    source: u8,
    pdu_specific: u8,
}

impl J1939Id {
    /// Decodes a J1939 identifier from an extended CAN identifier.
    ///
    /// # Arguments
    ///
    /// * `id` - the CAN identifier to decode.
    ///
    /// # Returns
    ///
    /// The decoded fields, or [`None`] if `id` is a standard identifier, which J1939 does
    /// not use.
    pub fn from_id(id: CanId) -> Option<J1939Id> {
        if !id.is_extended() {
            return None;
        }
        let raw = id.raw();
        let priority = ((raw >> 26) & 0x7) as u8;
        let page = (raw >> 24) & 0x3; // the EDP and DP bits together
        let pf = ((raw >> 16) & 0xFF) as u8;
        let ps = ((raw >> 8) & 0xFF) as u8;
        let source = (raw & 0xFF) as u8;
        let mut pgn = (page << 16) | (u32::from(pf) << 8);
        if pf >= PDU1_LIMIT {
            pgn |= u32::from(ps);
        }
        Some(J1939Id {
            priority,
            pgn,
            source,
            pdu_specific: ps,
        })
    }

    /// Composes a J1939 identifier from its fields.
    ///
    /// # Arguments
    ///
    /// * `priority` - the message priority, masked to its low three bits.
    /// * `pgn` - the parameter group number.
    /// * `source` - the source address.
    /// * `destination` - the destination address, used only for an addressed (PDU1)
    ///   parameter group and ignored for a broadcast (PDU2) one.
    ///
    /// # Returns
    ///
    /// The identifier fields.
    pub fn from_parts(priority: u8, pgn: u32, source: u8, destination: u8) -> J1939Id {
        let pf = ((pgn >> 8) & 0xFF) as u8;
        let pdu_specific = if pf < PDU1_LIMIT {
            destination
        } else {
            (pgn & 0xFF) as u8
        };
        J1939Id {
            priority: priority & 0x7,
            pgn: pgn & 0x3_FFFF,
            source,
            pdu_specific,
        }
    }

    /// Returns the message priority, 0 (highest) to 7.
    ///
    /// # Returns
    ///
    /// The priority.
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// Returns the parameter group number, which names what the message carries.
    ///
    /// # Returns
    ///
    /// The PGN.
    pub fn pgn(&self) -> u32 {
        self.pgn
    }

    /// Returns the source address: the node that sent the message.
    ///
    /// # Returns
    ///
    /// The source address.
    pub fn source(&self) -> u8 {
        self.source
    }

    /// Returns the PDU format byte of the parameter group.
    ///
    /// # Returns
    ///
    /// The PDU format.
    pub fn pdu_format(&self) -> u8 {
        ((self.pgn >> 8) & 0xFF) as u8
    }

    /// Returns the destination address, for an addressed message.
    ///
    /// # Returns
    ///
    /// The destination address for a PDU1 message, or [`None`] for a broadcast PDU2 one.
    pub fn destination(&self) -> Option<u8> {
        if self.pdu_format() < PDU1_LIMIT {
            Some(self.pdu_specific)
        } else {
            None
        }
    }

    /// Reports whether the message is a broadcast.
    ///
    /// # Returns
    ///
    /// `true` for a broadcast (PDU2) message, `false` for an addressed (PDU1) one.
    pub fn is_broadcast(&self) -> bool {
        self.pdu_format() >= PDU1_LIMIT
    }

    /// Composes the extended CAN identifier these fields describe.
    ///
    /// # Returns
    ///
    /// The extended [`CanId`].
    pub fn to_id(&self) -> CanId {
        let pf = self.pdu_format();
        let ps = if pf < PDU1_LIMIT {
            self.pdu_specific
        } else {
            (self.pgn & 0xFF) as u8
        };
        let page = (self.pgn >> 16) & 0x3;
        let raw = (u32::from(self.priority) << 26)
            | (page << 24)
            | (u32::from(pf) << 16)
            | (u32::from(ps) << 8)
            | u32::from(self.source);
        CanId::extended(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_standard_identifier_is_not_j1939() {
        assert_eq!(J1939Id::from_id(CanId::standard(0x100)), None);
    }

    #[test]
    fn the_engine_speed_broadcast_decodes() {
        let message = J1939Id::from_id(CanId::extended(0x0CF0_0400)).unwrap();
        assert_eq!(message.priority(), 3);
        assert_eq!(message.pgn(), 61_444);
        assert_eq!(message.source(), 0x00);
        assert_eq!(message.pdu_format(), 240);
        assert!(message.is_broadcast());
        assert_eq!(message.destination(), None);
    }

    #[test]
    fn an_addressed_request_decodes_its_destination() {
        // A request (PGN 59904, PDU format 0xEA) to address 0x21 from 0x01, priority 6.
        let message = J1939Id::from_id(CanId::extended(0x18EA_2101)).unwrap();
        assert_eq!(message.priority(), 6);
        assert_eq!(message.pgn(), 59_904);
        assert_eq!(message.source(), 0x01);
        assert!(!message.is_broadcast());
        assert_eq!(message.destination(), Some(0x21));
    }

    #[test]
    fn a_broadcast_identifier_round_trips() {
        let id = CanId::extended(0x0CF0_0400);
        assert_eq!(J1939Id::from_id(id).unwrap().to_id(), id);
    }

    #[test]
    fn an_addressed_identifier_round_trips() {
        let id = CanId::extended(0x18EA_2101);
        assert_eq!(J1939Id::from_id(id).unwrap().to_id(), id);
    }

    #[test]
    fn composing_an_addressed_message_places_the_destination() {
        let message = J1939Id::from_parts(6, 59_904, 0x01, 0x21);
        assert_eq!(message.to_id(), CanId::extended(0x18EA_2101));
        assert_eq!(message.destination(), Some(0x21));
    }

    #[test]
    fn composing_a_broadcast_message_ignores_the_destination() {
        // PGN 61444 is broadcast, so the destination argument has no effect.
        let with_dest = J1939Id::from_parts(3, 61_444, 0x00, 0x55);
        let without = J1939Id::from_parts(3, 61_444, 0x00, 0x00);
        assert_eq!(with_dest.to_id(), without.to_id());
        assert_eq!(with_dest.to_id(), CanId::extended(0x0CF0_0400));
    }
}
