use super::connection::Connection;

pub struct Manager;

impl Manager {
    pub fn init() {
        // Initialize connection to the X server
        let _conn = Connection::init();
    }
}
