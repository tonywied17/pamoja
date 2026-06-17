//! The CAN identifier: a standard 11-bit or extended 29-bit arbitration ID.

/// A CAN arbitration identifier.
///
/// CAN comes in two identifier widths: the original standard 11-bit form and the extended
/// 29-bit form that higher-layer protocols such as J1939 use to pack a priority, a
/// parameter group, and addresses into the ID itself. This type holds either, always
/// masked to its width.
///
/// # Examples
///
/// ```
/// use pamoja_can::CanId;
///
/// let std = CanId::standard(0x123);
/// assert!(!std.is_extended());
/// assert_eq!(std.raw(), 0x123);
///
/// // Values wider than the identifier are masked to fit.
/// assert_eq!(CanId::standard(0xFFFF).raw(), 0x7FF);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanId {
    bits: u32,
    extended: bool,
}

impl CanId {
    /// The mask of a standard 11-bit identifier.
    pub const STANDARD_MASK: u32 = 0x7FF;

    /// The mask of an extended 29-bit identifier.
    pub const EXTENDED_MASK: u32 = 0x1FFF_FFFF;

    /// Creates a standard 11-bit identifier, masking the value to fit.
    ///
    /// # Arguments
    ///
    /// * `raw` - the identifier value; bits above the low 11 are dropped.
    ///
    /// # Returns
    ///
    /// The identifier.
    pub fn standard(raw: u16) -> CanId {
        CanId {
            bits: u32::from(raw) & Self::STANDARD_MASK,
            extended: false,
        }
    }

    /// Creates an extended 29-bit identifier, masking the value to fit.
    ///
    /// # Arguments
    ///
    /// * `raw` - the identifier value; bits above the low 29 are dropped.
    ///
    /// # Returns
    ///
    /// The identifier.
    pub fn extended(raw: u32) -> CanId {
        CanId {
            bits: raw & Self::EXTENDED_MASK,
            extended: true,
        }
    }

    /// Returns the identifier value.
    ///
    /// # Returns
    ///
    /// The raw bits, already masked to the identifier's width.
    pub fn raw(&self) -> u32 {
        self.bits
    }

    /// Reports whether this is an extended 29-bit identifier.
    ///
    /// # Returns
    ///
    /// `true` for an extended identifier, `false` for a standard one.
    pub fn is_extended(&self) -> bool {
        self.extended
    }
}
