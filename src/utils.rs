use crate::{
    protocol::{Format, Screen},
    errors::ParseError,
};

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
pub trait Deserialize {
    /// Creates an instance of the implementing type from a given slice of bytes.
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

pub fn deserialize_into_vec<T: DeserializeList>(
    bytes: &[u8],
    n: u8,
) -> Result<(Vec<T>, &[u8]), ParseError> {
    // Calculate length of slice to split: size*n
    let size = T::size();
    let length = size * n;

    let mut formats: Vec<T> = Vec::new();

    // Split the slice by the calculated length.
    let result = match bytes.get(..length.into()) {
        Some(b) => b,
        None => return Err(ParseError::NotEnoughData),
    };

    // Iterate and deserialize each format within the LISTofFORMAT
    let mut start = 0;
    let mut end = size;
    for _ in 0..n {
        // TODO: use iterator instead.

        let slice = match result.get(start.into()..end.into()) {
            Some(b) => b,
            None => return Err(ParseError::NotEnoughData),
        };
        start += size;
        end += size;
        let format = T::deserialize(slice)?;
        formats.push(format)
    }
    Ok((formats, &bytes[length.into()..]))
}

pub trait DeserializeList {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized;
    /// Returns the size of an item in the list.
    fn size() -> u8;
}

impl DeserializeList for Format {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        // Each item in the LISTofFORMAT will have this byte format
        // 1  CARD8         depth
        // 1  CARD8         bits-per-pixel
        // 1  CARD8         scanline-pad
        // 5                unused
        let (depth, rest) = deserialize_into::<u8>(bytes)?;
        let (bits_per_pixel, rest) = deserialize_into::<u8>(rest)?;
        let (scanline_pad, rest) = deserialize_into::<u8>(rest)?;
        Ok(Format {
            depth,
            bits_per_pixel,
            scanline_pad,
        })
    }

    /// Each format within the LISTofFORMAT takes up 8 bytes of space.
    fn size() -> u8 {
        8
    }
}

impl DeserializeList for Screen {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn size() -> u8 {
        4  // TODO: this is not the actual value 
    }
}

/// Trims from the start of a byte slice to ensure alignment to
/// the specified boundary, based on a given length.
pub fn trim_by_padding(slice: &[u8], length: u16, boundary: u16) -> &[u8] {
    let p = length % boundary;
    if p > 0 {
        &slice[p.into()..]
    } else {
        slice
    }
}
