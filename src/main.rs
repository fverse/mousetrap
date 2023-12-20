use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::{env, process};

pub struct Connection;

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

// Opens a connection using the Unix domain sockets or over TCP
// Typically TCP connections are used for connecting to remote X11 server.
// So that we will first attempt to connect through Unix sockets.
// If that is unsuccessful, connect via TCP.
fn open(display_name: String) {
    // Parse address
    let conf = parse_conf(display_name);

    let mut connected: bool = false;
    let mut _stream;

    match UnixStream::connect(conf.socket_path) {
        Ok(_stream) => {
            connected = true;
            // Stream::Unix(Box::new(stream))
            // TODO: Handle stream
        }
        Err(err) => println!("Could not establish socket connection: {}", err),
    };

    if !connected {
        let addr: String = format!("{}:{}", conf.host, conf.port);
        println!("addr: {}", addr);
        _stream = match TcpStream::connect(addr) {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Could not establish connection: {}", e);
                process::exit(0);
            }
        };
    }
}

impl Connection {
    pub fn init() {
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
        open(display_name)
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
