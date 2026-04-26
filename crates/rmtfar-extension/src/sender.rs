//! UDP sender that forwards packets to the local bridge.

use rmtfar_protocol::BRIDGE_RECV_PORT;
use std::net::UdpSocket;

pub struct BridgeSender {
    socket: UdpSocket,
}

impl BridgeSender {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0")
            .expect("RMTFAR: Failed to bind sender UDP socket");
        socket
            .connect(format!("127.0.0.1:{BRIDGE_RECV_PORT}"))
            .expect("RMTFAR: Failed to connect sender to bridge");
        Self { socket }
    }

    pub fn send(&self, data: &[u8]) -> std::io::Result<()> {
        self.socket.send(data).map(|_| ())
    }
}
