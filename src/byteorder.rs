#[cfg(target_endian = "little")]
pub const BYTE_ORDER: u8 = b'l';
#[cfg(not(target_endian = "little"))]
pub const BYTE_ORDER: u8 = b'B';

pub struct BigEndian;

impl BigEndian {
    pub fn write_u16(&self, buf: &mut [u8], n: u16) {
        buf[..2].copy_from_slice(&n.to_be_bytes());
    }
}

pub struct LittleEndian;

impl LittleEndian {
    pub fn write_u16(&self, buf: &mut [u8], n: u16) {
        buf[..2].copy_from_slice(&n.to_le_bytes());
    }
}
