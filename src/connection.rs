use super::auth::XAuthEntry;
use crate::byteorder::BYTE_ORDER;
use std::io::{self, Error, Write};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::{env, fs, mem, process, ptr, slice};

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
pub struct ConnSetupResponse {
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
pub struct Failed {
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

/// Represents errors that can occur while attempting to establish a connection.
///
/// This enum is used to categorize the different types of connection errors
/// and provide more specific information about what went wrong during the
/// connection process.
pub enum ConnectionError {
    InvalidSocketPath,
    ConnectionRefused,
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
    pub fn new(entry: XAuthEntry) -> Self {
        Self {
            byte_order: BYTE_ORDER,
            protocol_major_version: PROTOCOL_MAJOR_VERSION,
            protocol_minor_version: PROTOCOL_MINOR_VERSION,
            authorization_protocol_data: entry.authorization_protocol_data,
            authorization_protocol_name: entry.authorization_protocol_name,
        }
    }

    fn byte_raw_slice(v: &Self) -> &[u8] {
        let p: *const Self = v;
        let p: *const u8 = p as *const u8;
        unsafe { slice::from_raw_parts(p, mem::size_of_val(v)) }
    }

    pub fn serialize(&self) -> &[u8] {
        let s: &[u8] = Self::byte_raw_slice(self);
        println!("slice: {:?}", s);
        s
    }
}

impl Stream {
    /// Opens a connection using the Unix domain sockets or over TCP
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

    /// Authenticate connection
    pub fn authenticate(&self) {
        let xauth_entries = XAuthEntry::parse().unwrap();

        // Construct the ConnSetupRequest
        let setup_request = ConnSetupRequest::new(xauth_entries[0].clone());

        let s = setup_request.serialize();

        // println!("xauth entries: {:?}", xauth_entries);

        // TODO: Send Authorization Details: If required, send the length of the authorization
        // protocol name and data, followed by the authorization protocol name (
        // e.g., MIT-MAGIC-COOKIE-1) and the authorization data (e.g., the cookie).

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
        let stream = Stream::open(display_name)?;

        // TODO: Authenticate
        stream.authenticate();

        // Write

        // Read

        Ok(Connection { stream })
    }
}
