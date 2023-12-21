use std::io::Error;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::{env, process};

pub struct Stream {
    variants: StreamVariants,
    open: bool,
}
pub enum StreamVariants {
    Tcp(TcpStream),
    Unix(UnixStream),
}

pub struct Connection {
    stream: Stream,
}

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

// Connects to the X11 server using socket path
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

// Connect to the X11  server via Tcp
fn connect_tcp(host: &str, port: &u16) -> std::io::Result<Stream> {
    let addr: String = format!("{}:{}", host, port);
    println!("addr: {}", addr);
    match TcpStream::connect(addr) {
        Ok(stream) => Ok(Stream {
            variants: StreamVariants::Tcp(stream),
            open: true,
        }),
        Err(e) => {
            eprintln!("Could not establish connection: {}", e);
            process::exit(0);
        }
    }
}

// Opens a connection using the Unix domain sockets or over TCP
// Typically TCP connections are used for connecting to remote X11 server.
// So that we will first attempt to connect through Unix sockets.
// If that is unsuccessful, connect via TCP.
fn open(display_name: String) -> Result<Connection, Error> {
    let XConf {
        socket_path,
        port,
        host,
        ..
    } = &parse_conf(display_name);

    // TODO: connect using abstract unix streams first
    let stream = match connect_unix_socket(socket_path) {
        Ok(stream) if stream.open => stream,
        _ => connect_tcp(host, port)?,
    };
    Ok(Connection { stream })
}

impl Connection {
    pub fn init() -> Result<Connection, Error> {
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

        // Opens a connection
        let conn = open(display_name)?;

        // TODO: Authenticate

        // Write

        // Read

        Ok(conn)
    }
}

pub struct Manager;

impl Manager {
    pub fn init() {
        // Initialize connection to the X server
        let _conn = Connection::init();
    }
}

fn main() {
    Manager::init()
}
