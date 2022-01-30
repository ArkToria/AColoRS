use std::net::TcpListener;

pub fn tcp_get_available_port(avoid: u16) -> Option<u16> {
    (11451..19198).find(|port| *port != avoid && tcp_port_is_available(*port))
}

pub fn tcp_port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}
