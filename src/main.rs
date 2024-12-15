use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;
    use tokio_tungstenite::client_async;

    #[tokio::test]
    async fn test_connect() {
        let socket = TcpStream::connect("127.0.0.1:8080")
            .await
            .expect("should connect");
        let (mut ws, _) = client_async("ws://127.0.0.1:8080", socket)
            .await
            .expect("should connect websocket");
        let data: [u8; 3] = [1, 2, 3];
        ws.send(Message::Binary(data.to_vec()))
            .await
            .expect("should send");
        let got = ws.next().await.unwrap().expect("should get message");
        assert_eq!(data.to_vec(), got.into_data());
    }

    #[tokio::test]
    async fn test_close_before_send() {
        let socket = TcpStream::connect("127.0.0.1:8080")
            .await
            .expect("should connect");
        let (mut ws, _) = client_async("ws://127.0.0.1:8080", socket)
            .await
            .expect("should connect websocket");
        ws.close(None).await.expect("should close");
    }

    #[tokio::test]
    async fn test_close_before_receive() {
        let socket = TcpStream::connect("127.0.0.1:8080")
            .await
            .expect("should connect");
        let (mut ws, _) = client_async("ws://127.0.0.1:8080", socket)
            .await
            .expect("should connect websocket");
        let data: [u8; 3] = [1, 2, 3];
        ws.send(Message::Binary(data.to_vec()))
            .await
            .expect("should send");
        ws.close(None).await.expect("should close");
    }
}

#[tokio::main]
async fn main() {
    let server_addr: SocketAddr = "0.0.0.0:8080".parse().expect("Invalid address");
    let listener = TcpListener::bind(&server_addr)
        .await
        .expect(&format!("Failed to bind to: {}", server_addr));
    println!("Listening on: {}", server_addr);
    loop {
        let (socket, addr) = match listener.accept().await {
            Ok((socket, addr)) => {
                println!("Accepted connection from {}", addr);
                (socket, addr)
            }
            Err(err) => {
                println!("Error accepting socket: {}", err);
                continue;
            }
        };
        tokio::spawn(async move {
            let mut ws = match accept_async(socket).await {
                Ok(ws) => ws,
                Err(err) => {
                    println!("Error accepting websocket from {addr}: {err}");
                    return;
                }
            };

            match ws.next().await {
                Some(Ok(msg)) => {
                    let msg = msg.into_data();
                    match ws.send(Message::Binary(msg)).await {
                        Err(err) => {
                            println!("Error sending to {addr}: {err}");
                            return;
                        }
                        _ => {}
                    }
                }
                Some(Err(err)) => {
                    println!("Error receiving message from {addr}: {err}");
                    return;
                }
                None => return, // Closed.
            }
            return;
        });
    }
}
