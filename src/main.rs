mod connection;
mod auth;
mod byteorder;
mod utils;
mod errors;
mod protocol;

use connection::Connection;

fn main() {
    let _conn = match Connection::init() {
        Ok(conn) => conn,
        Err(err) => {
            panic!("Connection failed: {}", err)
        }
    };
}
