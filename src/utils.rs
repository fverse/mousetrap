use crate::{
    errors::ParseError,
    protocol::{Format, Screen},
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
    if bytes.len() < length.into() {
        return Err(ParseError::NotEnoughData);
    }

    // Extract the relevant bytes for deserializing.
    let (result, rest) = bytes.split_at(length.into());

    Ok((String::from_utf8_lossy(result).to_string(), rest))
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
        if bytes.len() != 1 {
            return Err(ParseError::OutOfBound);
        }
        let result: [u8; 1] = bytes.try_into().map_err(|_| ParseError::Failed)?;
        Ok(u8::from_ne_bytes(result))
    }
}

impl Deserialize for u16 {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() != 2 {
            return Err(ParseError::OutOfBound);
        }
        let result = bytes.try_into().map_err(|_| ParseError::Failed)?;
        Ok(u16::from_ne_bytes(result))
    }
}

impl Deserialize for u32 {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() != 4 {
            return Err(ParseError::InvalidLength);
        }
        let result: [u8; 4] = bytes.try_into().map_err(|_| ParseError::Failed)?;
        Ok(u32::from_ne_bytes(result))
    }
}

impl Deserialize for u64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() != 8 {
            return Err(ParseError::InvalidLength);
        }
        let result: [u8; 8] = bytes.try_into().map_err(|_| ParseError::Failed)?;
        Ok(u64::from_ne_bytes(result))
    }
}

/// Deserialize a chunk of bytes into a vector of T
pub fn deserialize_into_vec<T: DeserializeList>(
    bytes: &[u8],
    n: usize,
) -> Result<(Vec<T>, &[u8]), ParseError> {
    // Calculate length of slice to split: element_size*n
    let element_size = T::size();
    let tot_length: usize = element_size
        .checked_mul(n.into())
        .ok_or(ParseError::OverFlow)?;

    if bytes.len() < tot_length {
        return Err(ParseError::NotEnoughData);
    }

    let mut formats: Vec<T> = Vec::with_capacity(n.into());

    // Split the slice by the calculated length.
    let (result, rest) = bytes.split_at(tot_length.into());

    // Iterate and deserialize each format within the LISTofFORMAT
    let mut start = 0 as usize;
    let mut end = element_size;
    for _ in 0..n {
        let slice: &[u8] = match result.get(start..end) {
            Some(b) => b,
            None => return Err(ParseError::NotEnoughData),
        };
        start += element_size;
        end += element_size;
        let format: T = T::deserialize(slice)?;
        formats.push(format)
    }
    Ok((formats, rest))
}

/// Returns the raw bytes of a T
pub fn byte_raw_slice<T>(v: &T) -> &[u8] {
    let p: *const T = v;
    let p: *const u8 = p as *const u8;
    unsafe { std::slice::from_raw_parts(p, std::mem::size_of_val(v)) }
}

pub trait DeserializeList {
    fn deserialize(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized;
    /// Returns the size of an item in the list.
    fn size() -> usize;
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
    fn size() -> usize {
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

    fn size() -> usize {
        4 // TODO: this is not the actual value
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
