//! UDP sender that delivers `RadioStateMessage` directly to the Mumble plugin.

use rmtfar_protocol::PLUGIN_RECV_PORT;
use std::net::UdpSocket;

pub struct PluginSender {
    socket: Option<UdpSocket>,
}

impl PluginSender {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0")
            .and_then(|s| {
                s.connect(format!("127.0.0.1:{PLUGIN_RECV_PORT}"))?;
                Ok(s)
            })
            .ok();
        Self { socket }
    }

    pub fn send(&self, data: &[u8]) -> std::io::Result<()> {
        match &self.socket {
            Some(s) => s.send(data).map(|_| ()),
            None => Err(std::io::Error::other("RMTFAR: plugin socket unavailable")),
        }
    }
}
