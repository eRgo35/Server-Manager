use sm_services::StatusProbe;
use tokio::net::TcpStream;

/// Reachability probe that opens a TCP connection, bounded by `timeout`.
pub struct TcpStatusProbe;

impl StatusProbe for TcpStatusProbe {
    fn is_up(
        &self,
        host: &str,
        port: u16,
        timeout: std::time::Duration,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            matches!(
                tokio::time::timeout(timeout, TcpStream::connect((host, port))).await,
                Ok(Ok(_))
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn up_when_listener_present() {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(TcpStatusProbe.is_up("127.0.0.1", port, Duration::from_secs(1)).await);
    }

    #[tokio::test]
    async fn down_on_closed_port() {
        assert!(!TcpStatusProbe.is_up("127.0.0.1", 1, Duration::from_millis(200)).await);
    }

    #[tokio::test]
    async fn down_on_unroutable_within_timeout() {
        let start = std::time::Instant::now();
        let up = TcpStatusProbe.is_up("10.255.255.1", 22, Duration::from_millis(300)).await;
        assert!(!up);
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}