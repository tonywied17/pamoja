//! A compact CBOR codec built on [`ciborium`].

use serde::de::DeserializeOwned;
use serde::Serialize;

use pamoja_core::{Error, Result};

use crate::Codec;

/// A codec that serializes values to and from CBOR.
///
/// CBOR is a compact, self-describing binary format, which makes it the default
/// choice for constrained devices and metered radio links where every byte costs
/// power or money. The codec works for any type that implements [`serde::Serialize`]
/// and [`serde::de::DeserializeOwned`].
///
/// # Examples
///
/// ```
/// use pamoja_codec::{CborCodec, Codec};
///
/// let codec = CborCodec;
/// let value = (1u8, "ok".to_owned());
/// let encoded = codec.encode(&value).unwrap();
/// let decoded: (u8, String) = codec.decode(&encoded).unwrap();
/// assert_eq!(decoded, value);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct CborCodec;

impl<T> Codec<T> for CborCodec
where
    T: Serialize + DeserializeOwned,
{
    fn encode(&self, value: &T) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        ciborium::into_writer(value, &mut buffer)
            .map_err(|error| Error::Codec(error.to_string()))?;
        Ok(buffer)
    }

    fn decode(&self, bytes: &[u8]) -> Result<T> {
        ciborium::from_reader(bytes).map_err(|error| Error::Codec(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_tuple() {
        let codec = CborCodec;
        let value = (42u32, true, vec![1u8, 2, 3]);
        let encoded = codec.encode(&value).expect("encode");
        let decoded: (u32, bool, Vec<u8>) = codec.decode(&encoded).expect("decode");
        assert_eq!(decoded, value);
    }

    #[test]
    fn decoding_truncated_input_is_a_codec_error() {
        let codec = CborCodec;
        let result: Result<(u32, u32)> = codec.decode(&[]);
        assert!(matches!(result, Err(Error::Codec(_))));
    }
}
