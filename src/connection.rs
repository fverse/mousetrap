use super::auth::XAuthEntry;
use crate::byteorder::BYTE_ORDER;
use crate::errors::{ConnectionError, ParseError};
use crate::utils::{deserialize_into, deserialize_into_string};
use std::io::{self, Error, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use std::{env, mem, process, slice};

/// Order of bits within the bytes for a Bitmap image.
pub enum BitOrder {
    LeastSignificant,
    MostSignificant,
}

pub struct Format {
    depth: u8,
    bits_per_pixel: u8,
    scanline_pad: u8,
}

const PROTOCOL_MAJOR_VERSION: u16 = 11;
const PROTOCOL_MINOR_VERSION: u16 = 0;

/// Unique identifier used for various things inside x11,
/// such as windows, pixmaps, fonts, [ColorMap]s and others.
pub struct XId(u32);

/// Identifier for a [VisualType].
type VisualId = u32;

/// A ColorMap consists of a set of entries defining color values.
type ColorMap = u32;

/// The numerical code of the key in a Keyboard.
type KeyCode = u8;

/// Family represents the protocol/address family
pub type Family = u16;

/// In x11, a Screen represents a physical display where Windows can be rendered.
/// So each struct fields in `Screen` represents various properties of the display.
pub struct Screen {
    /// Id of the root window.
    pub root: XId,
    /// Screen resolution width in pixels.
    pub width_in_px: u16,
    /// Screen resolution height in pixels.
    height_in_px: u16,
    /// Screen width in millimeters (Physical, I guess).
    width_in_mm: u16,
    /// Screen height in millimeters (Physical, I guess).
    height_in_mm: u16,
    /// The color depths that the screen supports.
    allowed_depths: Vec<Depth>,
    /// Default color depth of the root window.
    root_depth: u8,
    /// Id of [VisualType] for the root window.
    root_visual: VisualId,
    /// Default [ColorMap] of the Screen.
    default_colormap: ColorMap,
    /// The pixel values that correspond to white color on the screen.
    white_pixel: u32,
    // The pixel values that correspond to black color on the screen.
    black_pixel: u32,
    /// The minimum number of color maps that can be installed on the screen simultaneously.
    min_installed_maps: u16,
    /// The maximum number of color maps that can be installed on the screen simultaneously.
    max_installed_maps: u16,
    /// [BackingStore]
    backing_stores: BackingStore,
    /// Indicating whether the screen supports "save-under" functionality.
    /// This feature allows windows to automatically save and restore the
    /// area under them when they are moved or resized.
    save_unders: bool,
    /// A set of input events that the root window is currently set to report.
    /// This includes things like keyboard and mouse events.
    current_input_masks: u16, // SETofEVENT
}

// I don't fully understand what a backing-store is. But,
/// A backing-store of any of the `BackingStore` variants advises
/// the server when to maintain the contents of obscured regions.
pub enum BackingStore {
    Never,
    WhenMapped,
    Always,
    // TODO: If necessary, implement methods to convert from
    // and to the numeric representations (0, 1, 2) of these variants
    // Or we could simply use u8 for this instead of the enum
}

pub struct Depth {
    depth: u8,
    visuals: Vec<VisualType>,
}

pub struct VisualClass(u8);

impl VisualClass {
    pub const STATIC_GRAY: Self = Self(0);
    pub const STATIC_COLOR: Self = Self(1);
    pub const TRUE_COLOR: Self = Self(2);
    pub const GRAY_SCALE: Self = Self(3);
    pub const PSEUDO_COLOR: Self = Self(4);
    pub const DIRECT_COLOR: Self = Self(5);
}

/// VisualType describes the format of the pixel data in a window or image
pub struct VisualType {
    /// Unique Id of the visual type.
    visual_id: VisualId,
    class: VisualClass,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    bits_per_rgb_value: u8,
    colormap_entries: u16,
}

#[derive(Debug)]
pub struct ConnSetupRequest {
    byte_order: u8,
    /// Major protocol version supported by the server.
    protocol_major_version: u16,
    /// Minor protocol version supported by the  server.
    protocol_minor_version: u16,
    /// The authorization protocol the client expects the server to use (like MIT-MAGIC-COOKIE)
    authorization_protocol_name: Vec<u8>,
    /// The actual authorization value. ie, the cookie
    authorization_protocol_data: Vec<u8>,
}

/// Represents the response received from the x11 server if the connection is accepted.
pub struct ConnSetup {
    success: u8,
    /// Major protocol version supported by the server.
    protocol_major_version: u16,
    /// Minor protocol version supported by the  server.
    protocol_minor_version: u16,
    /// Vendor gives some identification of the owner of the server implementation.
    vendor: String,
    /// Release number of the x11 server.
    release_number: u32,
    /// Used by the client to generate resource IDs (like window IDs).
    resource_id_base: u32,
    /// Used by the client to generate resource IDs (like window IDs).
    resource_id_mask: u32,
    /// Byte order of the image data.
    image_byte_order: u8,
    /// Alignment requirements for bitmap data.
    bitmap_scanline_unit: u8,
    bitmap_scanline_pad: u8,
    /// Bit order within a byte of bitmap data.
    bitmap_bit_order: BitOrder,
    /// A list of supported formats for pixmap images.
    pixmap_formats: Vec<Format>,
    /// Screen(s) managed by the server.
    roots: Vec<Screen>,
    /// Size of the server's motion event buffer.
    motion_buffer_size: u32,
    /// The maximum length of a request that can be sent to the server.
    maximum_request_length: u16,
    /// The range of [KeyCode]s that are recognized by the server.
    min_keycode: KeyCode,
    max_keycode: KeyCode,
}

/// Represents the response received from the x11 server if the connection is refused.
pub struct ConnFailed {
    /// The connection status
    status: u8,
    /// Major and minor protocol version supported by the server.
    protocol_major_version: u8,
    protocol_minor_version: u8,
    /// Reason of failure.
    reason: String,
}

/// Stream is a wrapper for the `UnixStream` and `TcpStream`.
pub struct Stream {
    variants: StreamVariants,
    /// Indicates whether the Tcp or Socket connection is open.
    open: bool,
}

/// Variants of Stream
pub enum StreamVariants {
    Tcp(TcpStream),
    Unix(UnixStream),
}

pub struct Connection {
    stream: Stream,
}

pub struct ConnSetupResponse {
    buffer: Vec<u8>,
}

// Basic config variables for the x11 connection.
#[derive(Debug)]
struct XConf {
    display_number: u8,
    host: String,
    port: u16,
    socket_path: String,
}

fn parse_conf(display_name: String) -> XConf {
    let mut conf = XConf {
        display_number: 0,
        host: String::from("127.0.0.1"),
        port: 6000,
        socket_path: String::from("/tmp/.X11-unix/X"),
    };

    // Display number
    let display_number = display_name.trim_start_matches(":");
    conf.display_number = display_number.parse::<u8>().unwrap_or(0);

    // Port
    conf.port += conf.display_number as u16;

    // Socket path
    conf.socket_path.push_str(display_number);
    conf
}

impl ConnSetupRequest {
    /// Creates an instance of `ConnSetupRequest`
    pub fn new(entry: XAuthEntry) -> Self {
        Self {
            byte_order: BYTE_ORDER,
            protocol_major_version: PROTOCOL_MAJOR_VERSION,
            protocol_minor_version: PROTOCOL_MINOR_VERSION,
            authorization_protocol_data: entry.authorization_protocol_data,
            authorization_protocol_name: entry.authorization_protocol_name,
        }
    }

    /// Returns the raw bytes from a `ConnSetupRequest`
    fn byte_raw_slice(v: &Self) -> &[u8] {
        let p: *const Self = v;
        let p: *const u8 = p as *const u8;
        unsafe { slice::from_raw_parts(p, mem::size_of_val(v)) }
    }

    /// Converts an instance of `ConnSetupRequest` to x11 raw bytes
    //
    //  The connection setup request payload should be as follows:
    //   1 byte     Byte Order
    //   1 byte     Unused (Padding, for alignment)
    //   2 bytes    Protocol Major Version
    //   2 bytes    Protocol Minor Version
    //   2 bytes    Authorization protocol name length
    //   2 bytes    Authorization protocol data length
    //   2 bytes    Unused (Padding, for alignment)
    //   N bytes    Authorization protocol name
    //   P bytes    Unused (P = pad(N): To align the authorization protocol name to a 4-byte boundary)
    //   D bytes    Authorization protocol data
    //   Q bytes    Unused (Q = pad(D): To align the authorization protocol data to a 4-byte boundary)
    //
    //  TODO: think about rewriting the function to reduce code duplication.
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload: Vec<u8> = Vec::new();

        // Byte Order: 1 byte
        payload.extend_from_slice(&self.byte_order.to_ne_bytes());

        // Padding: 1 byte (For alignment. Unused)
        payload.extend_from_slice(&[0; 1]);

        // Protocol Major Version: 2 bytes
        payload.extend_from_slice(&self.protocol_major_version.to_ne_bytes());

        //  Protocol Minor Version: 2 bytes
        payload.extend_from_slice(&self.protocol_minor_version.to_ne_bytes());

        // Authorization protocol name length: 2 bytes
        payload.extend_from_slice(
            &u16::try_from(self.authorization_protocol_name.len())
                .unwrap()
                .to_ne_bytes(),
        );

        // Authorization protocol data length: 2 bytes
        payload.extend_from_slice(
            &u16::try_from(self.authorization_protocol_data.len())
                .unwrap()
                .to_ne_bytes(),
        );

        // Padding: 2 bytes (For alignment. Unused)
        payload.extend_from_slice(&[0; 2]);

        // Authorization Protocol name: N bytes
        payload.extend_from_slice(&self.authorization_protocol_name);

        // Padding: (To align the authorization protocol name to a 4-byte boundary. Unused)
        payload.extend_from_slice(&[0; 3][..(4 - (payload.len() % 4)) % 4]);

        // Authorization Protocol data: D bytes
        payload.extend_from_slice(&self.authorization_protocol_data);

        // Padding: (To align the authorization protocol name to a 4-byte boundary. Unused)
        payload.extend_from_slice(&[0; 3][..(4 - (payload.len() % 4)) % 4]);
        payload
    }
}


impl ConnSetup {
    pub fn parse_into(bytes: &[u8]) -> Result<ConnSetup, ConnectionError> {
        match bytes.get(0) {
            Some(0) => {
                // Connection failed
                // TODO: parse to ConnFailed
                Err(ConnectionError::ConnectionRefused)
            }
            Some(1) => {
                // Connection established
                // TODO: instead of unwrap, use ?
                return Ok(Self::from_bytes(bytes).unwrap());
            }
            Some(2) => {
                // Further authentication required
                // TODO: parse to AuthRequired
                Err(ConnectionError::FurtherAuthenticationRequired)
            }
            _ => Err(ConnectionError::InvalidResponseFromServer),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<ConnSetup, ParseError> {
        let (success, rest) = deserialize_into::<u8>(bytes)?;

        // Trim the unused 1 byte
        let rest = &rest[1..];
        let (protocol_major_version, rest) = deserialize_into::<u16>(rest)?;
        let (protocol_minor_version, rest) = deserialize_into::<u16>(rest)?;

        // 8+2n+(v+p+m)/4 : length in 4-byte units of "additional data"
        let (ln, rest) = deserialize_into::<u16>(rest)?;
        let (release_number, rest) = deserialize_into::<u32>(rest)?;
        let (resource_id_base, rest) = deserialize_into::<u32>(rest)?;
        let (resource_id_mask, rest) = deserialize_into::<u32>(rest)?;
        let (motion_buffer_size, rest) = deserialize_into::<u32>(rest)?;
        let (vendor_length, rest) = deserialize_into::<u16>(rest)?;
        let (maximum_request_length, rest) = deserialize_into::<u16>(rest)?;
        let (number_of_screens, rest) = deserialize_into::<u8>(rest)?;
        let (number_of_formats, rest) = deserialize_into::<u8>(rest)?;
        let (image_byte_order, rest) = deserialize_into::<u8>(rest)?;
        let (bitmap_bit_order, rest) = deserialize_into::<u8>(rest)?;
        let (bitmap_scanline_unit, rest) = deserialize_into::<u8>(rest)?;
        let (bitmap_scanline_pad, rest) = deserialize_into::<u8>(rest)?;
        let (min_keycode, rest) = deserialize_into::<u8>(rest)?;
        let (max_keycode, rest) = deserialize_into::<u8>(rest)?;
        
        // Trim the unused 4 bytes
        let rest: &[u8] = &rest[4..];
        let (vendor, rest) = deserialize_into_string(rest, vendor_length)?;

        todo!()
    }
}

impl Stream {
    /// Opens a connection to x11 server using the Unix domain sockets or over TCP.
    ///
    /// Typically TCP connections are used for connecting to remote X11 server.
    /// So that we will first attempt to connect through Unix sockets.
    /// If that is unsuccessful, connect via TCP.
    fn open(display_name: String) -> Result<Self, Error> {
        let XConf {
            socket_path,
            port,
            host,
            ..
        } = &parse_conf(display_name);

        // TODO: connect using abstract unix socket first
        let stream = match Self::connect_unix_socket(socket_path) {
            Ok(stream) if stream.open => stream,
            _ => Self::connect_tcp(host, port)?,
        };
        Ok(stream)
    }

    /// Connects to the X11 server using socket path
    fn connect_unix_socket(socket_path: &str) -> std::io::Result<Stream> {
        match UnixStream::connect(socket_path) {
            Ok(stream) => Ok(Stream {
                variants: StreamVariants::Unix(stream),
                open: true,
            }),
            Err(err) => {
                println!("Could not establish socket connection: {}", err);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Could not establish socket connection",
                ))
            }
        }
    }

    /// Connect to the X11  server via Tcp
    fn connect_tcp(host: &str, port: &u16) -> std::io::Result<Stream> {
        let addr: String = format!("{}:{}", host, port);
        println!("addr: {}", addr);
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(Stream {
                variants: StreamVariants::Tcp(stream),
                open: true,
            }),
            _ => {
                // eprintln!("Could not establish connection: {}", e);
                // process::exit(0);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Could not establish socket connection",
                ))
            }
        }
    }

    /// Writes to the stream
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if !self.open {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection not established yet",
            ));
        }
        match self.variants {
            StreamVariants::Tcp(ref mut stream) => stream.write(data),
            StreamVariants::Unix(ref mut stream) => stream.write(data),
        }
    }

    /// Reads from the stream
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.open {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection not established yet",
            ));
        }
        use std::io::Read;
        match self.variants {
            StreamVariants::Tcp(ref mut stream) => stream.read(buf),
            StreamVariants::Unix(ref mut stream) => stream.read(buf),
        }
    }

    /// Moves this stream into or out of non-blocking mode.
    pub fn set_nonblocking(&mut self, non_blocking: bool) -> std::io::Result<()> {
        match self.variants {
            StreamVariants::Tcp(ref mut stream) => stream.set_nonblocking(non_blocking),
            StreamVariants::Unix(ref mut stream) => stream.set_nonblocking(non_blocking),
        }
    }

    /// Authenticate connection
    pub fn authenticate(&mut self) {
        let xauth_entries = XAuthEntry::parse().unwrap();

        // Construct the ConnSetupRequest
        let setup_request = ConnSetupRequest::new(xauth_entries[0].clone());

        // Serialize the Setup Request
        let sr = setup_request.serialize();

        // Write the Connection Setup Request to the stream
        let mut written_count = 0;
        while written_count < sr.len() {
            match Self::write(self, &sr) {
                Ok(c) => written_count += c,
                Err(e) => {
                    eprintln!("An error occurred: {}", e);
                    process::exit(1);
                }
            }
        }

        println!("len: {}, written_count : {}", sr.len(), written_count);

        // Read server's connection setup response from the stream
        let mut buff = vec![0u8; 1000];

        Self::set_nonblocking(self, true); // TODO: handle error

        loop {
            match Self::read(self, &mut buff) {
                Ok(n) => {
                    println!("Read {} bytes", n);
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(1000));
                    continue;
                }
                Err(e) => {
                    eprintln!("An error occurred: {}", e);
                    process::exit(1);
                } // Handle other errors
            }
        }

        println!("read: {:?}", buff);

        // TODO: deserialize the bytes to `ConnSetup`
        let r = ConnSetup::parse_into(&buff);

        match self.variants {
            StreamVariants::Tcp(ref stream) => {
                println!("V: tcp");
            }
            StreamVariants::Unix(ref stream) => {
                println!("V: unix");
            }
        }
    }
}

impl Connection {
    pub fn init() -> Result<Self, Error> {
        let display_name = match env::var("DISPLAY") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => {
                eprintln!("Display not found");
                process::exit(0)
            }
            Err(env::VarError::NotUnicode(_)) => {
                eprintln!("The specified environment variable was found, but it did not contain valid unicode data.");
                process::exit(0)
            }
        };

        // Opens a connection stream
        let mut stream = Stream::open(display_name)?;

        // Authenticate the connection
        stream.authenticate();

        // TODO: This should be returned from the authenticate function
        Ok(Connection { stream })
    }
}
