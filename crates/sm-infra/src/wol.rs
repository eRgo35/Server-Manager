use sm_core::wol::magic_packet;
use sm_services::ServiceError;
use sm_services::Waker;
use tokio::net::UdpSocket;

pub struct UdpWaker;

impl UdpWaker {
    pub(crate) async fn wake_to(
        &self,
        mac: [u8; 6],
        addr: &str,
        ports: &[u16],
    ) -> Result<(), ServiceError> {
        // Bind to 0.0.0.0:0 to send from any interface with any available port
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| ServiceError::Network(e.to_string()))?;

        // Enable broadcast mode
        socket
            .set_broadcast(true)
            .map_err(|e| ServiceError::Network(e.to_string()))?;

        let packet = magic_packet(mac);

        // Send magic packet to each WOL port
        for &port in ports {
            let target = format!("{}:{}", addr, port);
            socket
                .send_to(&packet, &target)
                .await
                .map_err(|e| ServiceError::Network(e.to_string()))?;
        }

        Ok(())
    }
}

impl Waker for UdpWaker {
    fn wake(
        &self,
        mac: [u8; 6],
        broadcast_addr: &str,
    ) -> impl std::future::Future<Output = Result<(), ServiceError>> + Send {
        self.wake_to(mac, broadcast_addr, &sm_core::wol::WOL_PORTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wake_to_sends_magic_packet() {
        // Bind a test socket on localhost:0 to get an available port
        let test_socket = UdpSocket::bind("127.0.0.1:0")
            .await
            .expect("failed to bind test socket");
        let test_addr = test_socket.local_addr().expect("failed to get local addr");
        let test_port = test_addr.port();

        // Create a waker and send a packet to our test socket
        let waker = UdpWaker;
        let mac = [1, 2, 3, 4, 5, 6];

        waker
            .wake_to(mac, "127.0.0.1", &[test_port])
            .await
            .expect("wake_to failed");

        // Receive the packet
        let mut buf = [0u8; 102];
        let (n, _) = test_socket
            .recv_from(&mut buf)
            .await
            .expect("recv_from failed");

        // Verify packet size (should be 102 bytes)
        assert_eq!(n, 102, "received packet size should be 102 bytes");

        // Verify first 6 bytes are 0xFF (the broadcast prefix)
        assert_eq!(
            &buf[0..6],
            &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            "first 6 bytes should be 0xFF"
        );

        // Verify the MAC address is repeated in the packet
        assert_eq!(&buf[6..12], &mac, "MAC should appear at offset 6");
    }
}
