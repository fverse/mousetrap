use crate::errors::ParseError;

/// Parses a given byte slice into a value of type `T` and returns the parsed value along with the remaining slice.
///
/// If the input slice contains not enough bytes to parse a value of the specified type `T`,
/// it returns a `ParseError::NotEnoughData` error.
///
/// TODO: The same can be achieved using macros more gracefully i guess. Consider doing that way.
pub fn deserialize_into<T: Deserialize>(bytes: &[u8]) -> Result<(T, &[u8]), ParseError> {
    let size = core::mem::size_of::<T>();
    // Extract the relevant bytes for deserializing.
    let result = match bytes.get(..size) {
        Some(b) => b,
        None => return Err(ParseError::NotEnoughData),
    };
    Ok((T::deserialize(result)?, &bytes[size..]))
}

/// Parses a given byte slice into a String and returns the parsed value along with the remaining slice.
///
/// If the input slice contains not enough bytes to parse,
/// it returns a `ParseError::NotEnoughData` error.
pub fn deserialize_into_string(bytes: &[u8], length: u16) -> Result<(String, &[u8]), ParseError> {
    // Extract the relevant bytes for deserializing.
    let result = match bytes.get(..length.into()) {
        Some(b) => b,
        None => return Err(ParseError::NotEnoughData),
    };
    Ok((
        String::from_utf8_lossy(result.try_into().unwrap()).to_string(),
        &bytes[length.into()..],
    ))
}

/// A trait for parsing an instance of a type from a byte slice.
///
/// This trait defines a single method, `deserialize`, which attempts to create an instance
/// of the implementing type from a given slice of bytes.
pub trait Deserialize {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized;
}

impl Deserialize for u8 {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() == 1 {
            Ok(u8::from_ne_bytes(bytes.try_into().unwrap()))
        } else {
            Err(ParseError::OutOfBound)
        }
    }
}

impl Deserialize for u16 {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() == 2 {
            Ok(u16::from_ne_bytes(bytes.try_into().unwrap()))
        } else {
            Err(ParseError::OutOfBound)
        }
    }
}

impl Deserialize for u32 {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() == 4 {
            Ok(u32::from_ne_bytes(bytes.try_into().unwrap()))
        } else {
            Err(ParseError::OutOfBound)
        }
    }
}

impl Deserialize for u64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() == 8 {
            Ok(u64::from_ne_bytes(bytes.try_into().unwrap()))
        } else {
            Err(ParseError::OutOfBound)
        }
    }
}

