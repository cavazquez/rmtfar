//! UDP sender that forwards packets to the local bridge.
//!
//! Uses `Option<UdpSocket>` instead of panicking so that a failure to bind
//! (e.g. firewall, ephemeral ports exhausted) does not crash Arma 3.

use rmtfar_protocol::BRIDGE_RECV_PORT;
use std::net::UdpSocket;

pub struct BridgeSender {
    socket: Option<UdpSocket>,
}

impl BridgeSender {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0")
            .and_then(|s| {
                s.connect(format!("127.0.0.1:{BRIDGE_RECV_PORT}"))?;
                Ok(s)
            })
            .ok();
        Self { socket }
    }

    /// Returns `Err` if the socket was never created or `send` failed.
    pub fn send(&self, data: &[u8]) -> std::io::Result<()> {
        match &self.socket {
            Some(s) => s.send(data).map(|_| ()),
            None => Err(std::io::Error::other("RMTFAR: bridge socket unavailable")),
        }
    }
}
