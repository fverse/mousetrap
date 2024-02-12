/// Order of bits within the bytes for a Bitmap image.
pub enum BitOrder {
    LeastSignificant,
    MostSignificant,
}

#[derive(Debug)]
pub struct Format {
    pub depth: u8,
    pub bits_per_pixel: u8,
    pub scanline_pad: u8,
}

pub const PROTOCOL_MAJOR_VERSION: u16 = 11;
pub const PROTOCOL_MINOR_VERSION: u16 = 0;

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
    pub byte_order: u8,
    /// Major protocol version supported by the server.
    pub protocol_major_version: u16,
    /// Minor protocol version supported by the  server.
    pub protocol_minor_version: u16,
    /// The authorization protocol the client expects the server to use (like MIT-MAGIC-COOKIE)
    pub authorization_protocol_name: Vec<u8>,
    /// The actual authorization value. ie, the cookie
    pub authorization_protocol_data: Vec<u8>,
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
