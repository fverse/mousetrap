mod connection;
mod auth;
mod byteorder;

use connection::Connection;

fn main() {
    let _conn = match Connection::init() {
        Ok(conn) => conn,
        Err(err) => {
            panic!("Connection failed: {}", err)
        }
    };
}
