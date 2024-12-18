use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Clone)]
enum Turn {
    Black,
    White,
}

struct Checkers {
    turn: Turn,
    board: [[char; 8]; 8],
}

impl Checkers {
    fn new() -> Checkers {
        return Checkers::from(
            concat!(
                ".w.w.w.w\n",
                "w.w.w.w.\n",
                ".w.w.w.w\n",
                "........\n",
                "........\n",
                "b.b.b.b.\n",
                ".b.b.b.b\n",
                "b.b.b.b.\n"
            ),
            Turn::Black,
            8,
        );
    }

    fn from(s: &str, t: Turn, num_cols: usize) -> Checkers {
        let mut ch = Checkers {
            turn: t,
            board: [['.'; 8]; 8],
        };

        assert_eq!(s.len() % num_cols, 0);
        let num_rows = s.len() / num_cols;

        for i in 0..num_rows {
            for j in 0..num_cols {
                ch.board[i][j] = s.chars().nth(i * num_cols + j).unwrap();
            }
        }
        return ch;
    }

    fn can_move_to(&self, src: (usize, usize), dst: (usize, usize)) -> bool {
        return false;
    }

    fn has_opponent(turn: Turn, vboard: &[[char; 8]; 8], i: isize, j: isize) -> bool {
        if i < 0 || j < 0 || i > 7 || j > 7 {
            return false;
        }
        let i = i as usize;
        let j = j as usize;
        return match turn {
            Turn::Black => vboard[i][j] == 'b' || vboard[i][j] == 'B',
            Turn::White => vboard[i][j] == 'w' || vboard[i][j] == 'W',
        };
    }
    fn is_empty(vboard: &[[char; 8]; 8], i: isize, j: isize) -> bool {
        if i < 0 || j < 0 || i > 7 || j > 7 {
            return false;
        }
        let i = i as usize;
        let j = j as usize;
        return vboard[i][j] == '.';
    }

    fn get_moves_recursive(
        turn: Turn,
        piece: char,
        i: isize,
        j: isize,
        moves: &mut Vec<Move>,
        vboard: &mut [[char; 8]; 8],
    ) {
        match turn {
            Turn::Black => match piece {
                'b' | 'B' => {}
                _ => return,
            },
            Turn::White => match piece {
                'w' | 'W' => {}
                _ => return,
            },
        }

        match piece {
            'b' => {
                // Check NW and NE.
                if Self::is_empty(vboard, i - 1, j - 1) {
                    moves.push(Move {
                        steps: vec![Step {
                            src: (i as u8, j as u8),
                            dst: ((i - 1) as u8, (j - 1) as u8),
                            capture: false,
                        }],
                    });
                }

                if Self::is_empty(vboard, i - 1, j + 1) {
                    moves.push(Move {
                        steps: vec![Step {
                            src: (i as u8, j as u8),
                            dst: ((i - 1) as u8, (j + 1) as u8),
                            capture: false,
                        }],
                    });
                }
            }
            'B' => {
                // Check NW, NE, SW, SE.
            }
            'w' => {
                // Check SW and SE.
            }
            'W' => {
                // Check SW, SE, NW, NE.
            }
            _ => panic!("Unexpected"),
        }
    }

    fn get_moves(&self, i: isize, j: isize) -> Vec<Move> {
        assert!(!(i < 0 || j < 0 || i > 7 || j > 7));
        let mut moves = Vec::<Move>::new();
        let mut vboard = self.board;
        // Remove self.
        let piece = vboard[i as usize][j as usize];
        vboard[i as usize][j as usize] = '.';
        Self::get_moves_recursive(self.turn.clone(), piece, i, j, &mut moves, &mut vboard);
        return moves;
    }

    fn make_move(&self, mv: Move) {
        assert!(false);
    }
}

#[derive(Debug, PartialEq)]
struct Step {
    src: (u8, u8),
    dst: (u8, u8),
    capture: bool,
}

// A Move may be a sequence of steps. Commonly it is a single step.
#[derive(Debug, PartialEq)]
struct Move {
    steps: Vec<Step>,
}

impl std::fmt::Display for Checkers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..self.board.len() {
            for j in 0..self.board[i].len() {
                write!(f, "{}", self.board[i][j] as char)?;
            }
            write!(f, "\n")?;
        }
        return std::fmt::Result::Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;
    use tokio_tungstenite::client_async;

    macro_rules! assert_boardequal {
        ($a:ident, $b:ident) => {
            assert_eq!($a, $b, "{}!=\n{}", $a, $b);
        };
    }

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

    #[tokio::test]
    async fn can_display_board() {
        let c = Checkers::new();
        let got = format!("{}", c);
        let expect = concat!(
            ".w.w.w.w\n",
            "w.w.w.w.\n",
            ".w.w.w.w\n",
            "........\n",
            "........\n",
            "b.b.b.b.\n",
            ".b.b.b.b\n",
            "b.b.b.b.\n"
        );
        assert_boardequal!(got, expect);
    }

    #[tokio::test]
    async fn finds_moves() {
        let c = Checkers::from(
            concat!(
                "...", //
                ".b.", //
            ),
            Turn::Black,
            3,
        );

        let moves = c.get_moves(1, 1);
        assert_eq!(
            moves,
            vec![
                Move {
                    steps: vec![Step {
                        src: (1, 1),
                        dst: (0, 0),
                        capture: false
                    }]
                },
                Move {
                    steps: vec![Step {
                        src: (1, 1),
                        dst: (0, 2),
                        capture: false
                    }]
                }
            ]
        );
    }

    #[tokio::test]
    async fn determines_capture() {
        let c = Checkers::from(
            concat!(
                "...", //
                ".w.", //
                "..b", //
            ),
            Turn::Black,
            3,
        );

        let moves = c.get_moves(7, 0);
        assert_eq!(
            moves,
            vec![Move {
                steps: vec![Step {
                    src: (2, 2),
                    dst: (0, 0),
                    capture: true
                }]
            }]
        );
    }

    #[tokio::test]
    async fn requires_capture() {
        let c = Checkers::from(
            concat!(
                "....", //
                ".w..", //
                "..b.", //
            ),
            Turn::Black,
            4,
        );

        let moves = c.get_moves(7, 0);
        assert_eq!(
            moves,
            vec![Move {
                steps: vec![Step {
                    src: (2, 2),
                    dst: (0, 0),
                    capture: true
                }]
            }]
        );
    }

    #[tokio::test]
    async fn determines_capture_sequence() {
        let c = Checkers::from(
            concat!(
                "....", //
                ".w..", //
                "....", //
                ".w..", //
                "..b.", //
            ),
            Turn::Black,
            4,
        );

        let moves = c.get_moves(7, 0);
        assert_eq!(
            moves,
            vec![Move {
                steps: vec![
                    Step {
                        src: (4, 2),
                        dst: (2, 0),
                        capture: true
                    },
                    Step {
                        src: (2, 0),
                        dst: (0, 2),
                        capture: true
                    }
                ]
            }]
        );
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
