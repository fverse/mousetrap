use super::connection::Family;
use std::env::var_os;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

/// XAuthEntry represents an entry in the .Xauthority file.
/// It is a binary file consisting of a sequence of entries.
#[derive(Debug, Clone)]
pub struct XAuthEntry {
    /// The protocol family.
    pub family: Family,
    /// Host address
    pub address: Vec<u8>,
    pub display_number: Vec<u8>,
    /// The authorization protocol the client expects the server to use (like MIT-MAGIC-COOKIE)
    pub authorization_protocol_name: Vec<u8>,
    /// The actual authorization value: the cookie
    pub authorization_protocol_data: Vec<u8>,
}

impl XAuthEntry {

    /// Parse the .Xauthority file
    pub fn parse() -> io::Result<Vec<XAuthEntry>> {
        
        // Open .Xauthority file
        let xauth_file = open()?;
        let mut reader = io::BufReader::new(xauth_file);
        let mut xauth_entries = Vec::new();

        /// Reads 2 bytes from the provided reader and returns them as a u16 value. 
        /// Used to read length fields (i.e. 2-byte values) that precede data 
        /// sections in the .Xauthority file.
        fn read_preceding_bytes<R: Read>(reader: &mut R) -> io::Result<u16> {
            let mut buf = [0u8; 2];
            match reader.read_exact(&mut buf) {
                Err(err) => Err(err),
                _ => Ok(u16::from_be_bytes(buf)),
            }
        }

        /// Calls read_preceding_bytes to first determine the length of the subsequent 
        /// data section, then reads that many bytes. 
        /// Used for reading variable-length data like address, display number, 
        /// authorization name, and authorization data.
        fn read_subsequent_bytes<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
            let len = read_preceding_bytes(reader)? as usize;
            let mut buf = vec![0u8; len];
            match reader.read_exact(&mut buf) {
                Err(err) => Err(err),
                _ => Ok(buf),
            }
        }

        /// Reads a single entry from the .Xauthority file.
        //
        //  The sequence of entries in the .Xauthority file is in the following order:
        //    2 bytes	Family value (second byte is as in protocol HOST)
        //    2 bytes	address length (always MSB first)
        //    A bytes	host address (as in protocol HOST)
        //    2 bytes	display "number" length (always MSB first)
        //    S bytes    display "number" string
        //    2 bytes	name length (always MSB first)
        //    N bytes	authorization name string
        //    2 bytes	data length (always MSB first)
        //    D bytes	authorization data string
        fn read_xauth_entry<R: Read>(reader: &mut R) -> std::io::Result<XAuthEntry> {
          
            let family = read_preceding_bytes(reader)?;
            let address = read_subsequent_bytes(reader)?;
            let display_number = read_subsequent_bytes(reader)?;
            let auth_name = read_subsequent_bytes(reader)?;
            let auth_data = read_subsequent_bytes(reader)?;

            Ok(XAuthEntry {
                family,
                address,
                display_number,
                authorization_protocol_name: auth_name,
                authorization_protocol_data: auth_data,
            })
        }

        while let Ok(record) = read_xauth_entry(&mut reader) {
            xauth_entries.push(record);
        }
        Ok(xauth_entries)
    }

}

/// Get XAUTHORITY file path
pub fn get_xauth_filename() -> Option<PathBuf> {
    // TODO: check in home directory
    match var_os("XAUTHORITY") {
        Some(p) => Some(p.into()),
        None => None,
    }
}

/// Open Xauthority file
pub fn open() -> std::io::Result<File> {
    if let Some(path) = get_xauth_filename() {
        match File::open(path) {
            Ok(f) => Ok(f),
            Err(e) => {
                println!("File open err: {}", e);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed opening file",
                ))
            }
        }
    } else {
        // if !Path::new(&path).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get XAUTHORITY environment variable: The variable may not be set",
        ));
        // }
    }
}
