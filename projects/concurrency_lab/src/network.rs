//! Local TCP example: real socket I/O with bounded async clients.

use crate::map_bounded;
use std::error::Error;
use std::io;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;

/// Start an ephemeral local server, send each input over TCP, and return its
/// square. At most `limit` clients are active. The server accepts exactly the
/// number of requests supplied; this is a teaching example, not a daemon.
pub async fn run_local_tcp_demo(
    inputs: Vec<u32>,
    limit: usize,
) -> Result<Vec<u32>, Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let request_count = inputs.len();
    let server = tokio::spawn(async move {
        let mut handlers = JoinSet::new();
        for _ in 0..request_count {
            let (mut stream, _) = listener.accept().await?;
            handlers.spawn(async move {
                let mut request = [0_u8; 4];
                stream.read_exact(&mut request).await?;
                let value = u32::from_be_bytes(request);
                let square = value.checked_mul(value).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "square overflows u32")
                })?;
                stream.write_all(&square.to_be_bytes()).await?;
                Ok::<(), io::Error>(())
            });
        }
        while let Some(result) = handlers.join_next().await {
            result??;
        }
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    let clients = map_bounded(inputs, limit, move |value| async move {
        tokio::time::timeout(Duration::from_secs(2), async {
            let mut stream = TcpStream::connect(address).await?;
            stream.write_all(&value.to_be_bytes()).await?;
            let mut response = [0_u8; 4];
            stream.read_exact(&mut response).await?;
            Ok::<u32, io::Error>(u32::from_be_bytes(response))
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "local request timed out"))?
    })
    .await;

    let responses = match clients {
        Ok(responses) => responses,
        Err(error) => {
            server.abort();
            return Err(error.into());
        }
    };
    let values = match responses.into_iter().collect::<io::Result<Vec<_>>>() {
        Ok(values) => values,
        Err(error) => {
            server.abort();
            return Err(error.into());
        }
    };
    server.await??;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_tcp_requests_keep_input_order() {
        let values = tokio::time::timeout(
            Duration::from_secs(5),
            run_local_tcp_demo(vec![5, 1, 4, 2, 3], 2),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(values, vec![25, 1, 16, 4, 9]);
    }
}
