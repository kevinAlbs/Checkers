use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Turn {
    Black,
    White,
}

struct Checkers {
    turn: Turn,
    board: [[char; 8]; 8],
    in_step: bool,
}

fn make_board(s: &str, num_cols: usize) -> [[char; 8]; 8] {
    let mut board = [['.'; 8]; 8];

    assert_eq!(s.len() % num_cols, 0);
    let num_rows = s.len() / num_cols;

    for i in 0..num_rows {
        for j in 0..num_cols {
            board[i][j] = s.chars().nth(i * num_cols + j).unwrap();
        }
    }
    return board;
}

fn board_to_string(b: &[[char; 8]; 8]) -> String {
    let mut as_str = String::new();
    for row in b {
        for square in row {
            as_str.push(*square);
        }
        as_str.push('\n');
    }
    return as_str;
}

impl Checkers {
    fn new() -> Checkers {
        return Checkers::from(
            concat!(
                ".w.w.w.w", //
                "w.w.w.w.", //
                ".w.w.w.w", //
                "........", //
                "........", //
                "b.b.b.b.", //
                ".b.b.b.b", //
                "b.b.b.b.", //
            ),
            Turn::Black,
            8,
        );
    }

    fn from(s: &str, t: Turn, num_cols: usize) -> Checkers {
        let mut ch = Checkers {
            turn: t,
            board: make_board(s, num_cols),
            in_step: false,
        };

        return ch;
    }

    fn can_move_to(&self, src: (usize, usize), dst: (usize, usize)) -> bool {
        return false;
    }

    fn is_opponent(turn: Turn, vboard: &[[char; 8]; 8], i: isize, j: isize) -> bool {
        if i < 0 || j < 0 || i > 7 || j > 7 {
            return false;
        }
        let i = i as usize;
        let j = j as usize;
        return match turn {
            Turn::Black => vboard[i][j] == 'w' || vboard[i][j] == 'W',
            Turn::White => vboard[i][j] == 'b' || vboard[i][j] == 'B',
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

    fn get_jumps_recursive(
        turn: Turn,
        piece: char,
        i: isize,
        j: isize,
        moves: &mut Vec<Move>,
        steps: &mut Vec<Step>,
        vboard: &mut [[char; 8]; 8],
    ) {
        let mut maybe_add = |src_i: isize,
                             src_j: isize,
                             jump_i: isize,
                             jump_j: isize,
                             dst_i: isize,
                             dst_j: isize| {
            if Self::is_opponent(turn, &vboard, jump_i, jump_j) {
                let jumped_piece = vboard[jump_i as usize][jump_j as usize];
                steps.push(Step {
                    src: (src_i as u8, src_j as u8),
                    dst: (dst_i as u8, dst_j as u8),
                    capture: Some((jump_i as u8, jump_j as u8)),
                });

                // Remove from vboard.
                vboard[jump_i as usize][jump_j as usize] = '.';

                // Recurse.
                Self::get_jumps_recursive(turn, piece, dst_i, dst_j, moves, steps, vboard);

                // Restore board.
                steps.pop().unwrap();
                vboard[jump_i as usize][jump_j as usize] = jumped_piece;
                return true;
            }
            return false;
        };

        match piece {
            'b' => {
                // Check NW and NE.
                let has_any_jumps = maybe_add(i, j, i - 1, j - 1, i - 2, j - 2)
                    || maybe_add(i, j, i - 1, j + 1, i - 2, j + 2);
                if !has_any_jumps && steps.len() > 0 {
                    moves.push(Move {
                        steps: steps.clone(),
                    });
                }
            }
            'w' => {
                // Check SW and SE.
                let has_any_jumps = maybe_add(i, j, i + 1, j - 1, i + 2, j - 2)
                    || maybe_add(i, j, i + 1, j + 1, i + 2, j + 2);
                if !has_any_jumps && steps.len() > 0 {
                    moves.push(Move {
                        steps: steps.clone(),
                    });
                }
            }
            'B' | 'W' => {
                // Check NW, NE, SW, SE.
                let has_any_jumps = maybe_add(i, j, i - 1, j - 1, i - 2, j - 2)
                    || maybe_add(i, j, i - 1, j + 1, i - 2, j + 2)
                    || maybe_add(i, j, i + 1, j - 1, i + 2, j - 2)
                    || maybe_add(i, j, i + 1, j + 1, i + 2, j + 2);

                if !has_any_jumps && steps.len() > 0 {
                    moves.push(Move {
                        steps: steps.clone(),
                    });
                }
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

        match self.turn {
            Turn::Black => match piece {
                'b' | 'B' => {}
                _ => return moves,
            },
            Turn::White => match piece {
                'w' | 'W' => {}
                _ => return moves,
            },
        }

        let mut maybe_add = |src_i: isize, src_j: isize, dst_i: isize, dst_j: isize| {
            if Self::is_empty(&vboard, dst_i, dst_j) {
                moves.push(Move {
                    steps: vec![Step {
                        src: (src_i as u8, src_j as u8),
                        dst: (dst_i as u8, dst_j as u8),
                        capture: None,
                    }],
                });
            }
        };

        match piece {
            'b' => {
                // Check NW and NE.
                maybe_add(i, j, i - 1, j - 1);
                maybe_add(i, j, i - 1, j + 1);
            }
            'w' => {
                // Check SW and SE.
                maybe_add(i, j, i + 1, j - 1);
                maybe_add(i, j, i + 1, j + 1);
            }
            'B' | 'W' => {
                // Check NW, NE, SW, SE.
                maybe_add(i, j, i - 1, j - 1);
                maybe_add(i, j, i - 1, j + 1);
                maybe_add(i, j, i + 1, j - 1);
                maybe_add(i, j, i + 1, j + 1);
            }
            _ => panic!("Unexpected"),
        }

        let mut steps = Vec::<Step>::new();
        Self::get_jumps_recursive(
            self.turn.clone(),
            piece,
            i,
            j,
            &mut moves,
            &mut steps,
            &mut vboard,
        );

        // If there are any moves with a capture, remove non-capture moves.
        let mut has_capture = false;
        for m in &moves {
            if m.steps[0].capture.is_some() {
                has_capture = true;
                break;
            }
        }

        if has_capture {
            // Keep only moves that have a capture.
            moves.retain(|el| {
                return el.steps[0].capture.is_some();
            });
        }
        return moves;
    }

    fn make_step(&mut self, s: Step) {
        let moves = self.get_moves(s.src.0 as isize, s.src.1 as isize);
        let mut found = false;
        for ref m in moves {
            if m.steps[0] == s {
                found = true;
                break;
            }
        }
        assert!(found);
        let piece = self.board[s.src.0 as usize][s.src.1 as usize];
        self.board[s.src.0 as usize][s.src.1 as usize] = '.';
        assert_eq!(self.board[s.dst.0 as usize][s.dst.1 as usize], '.');
        self.board[s.dst.0 as usize][s.dst.1 as usize] = piece;
        if s.capture.is_some() {
            let jump_i = s.capture.unwrap().0;
            let jump_j = s.capture.unwrap().0;
            assert!(Self::is_opponent(
                self.turn,
                &self.board,
                jump_i as isize,
                jump_j as isize
            ));
            self.board[jump_i as usize][jump_j as usize] = '.';
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Step {
    src: (u8, u8),
    dst: (u8, u8),
    capture: Option<(u8, u8)>,
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
        ($a:expr, $b:expr) => {
            let a_str = board_to_string($a);
            let b_str = board_to_string($b);
            assert_eq!(a_str, b_str, "\n{}!=\n{}", a_str, b_str);
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
        let expect = make_board(
            concat!(
                ".w.w.w.w", //
                "w.w.w.w.", //
                ".w.w.w.w", //
                "........", //
                "........", //
                "b.b.b.b.", //
                ".b.b.b.b", //
                "b.b.b.b.", //
            ),
            8,
        );
        assert_boardequal!(&c.board, &expect);
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
                        capture: None
                    }]
                },
                Move {
                    steps: vec![Step {
                        src: (1, 1),
                        dst: (0, 2),
                        capture: None
                    }]
                }
            ]
        );
    }

    #[tokio::test]
    async fn determines_capture() {
        let c = Checkers::from(
            concat!(
                "....", //
                ".w..", //
                "..b.", //
            ),
            Turn::Black,
            4,
        );

        let moves = c.get_moves(2, 2);
        assert_eq!(
            moves,
            vec![Move {
                steps: vec![Step {
                    src: (2, 2),
                    dst: (0, 0),
                    capture: Some((1, 1))
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

        let moves = c.get_moves(2, 2);
        assert_eq!(
            moves,
            vec![Move {
                steps: vec![Step {
                    src: (2, 2),
                    dst: (0, 0),
                    capture: Some((1, 1))
                }]
            }]
        );
    }

    #[tokio::test]
    async fn determines_capture_sequence() {
        let c = Checkers::from(
            concat!(
                "....", //
                ".W..", //
                "....", //
                ".w..", //
                "..b.", //
            ),
            Turn::Black,
            4,
        );

        let moves = c.get_moves(4, 2);
        assert_eq!(
            moves,
            vec![Move {
                steps: vec![
                    Step {
                        src: (4, 2),
                        dst: (2, 0),
                        capture: Some((3, 1))
                    },
                    Step {
                        src: (2, 0),
                        dst: (0, 2),
                        capture: Some((1, 1))
                    }
                ]
            }]
        );
    }

    #[tokio::test]
    async fn can_step() {
        let mut c = Checkers::from(
            concat!(
                "...", //
                ".b.", //
            ),
            Turn::Black,
            3,
        );

        c.make_step(Step {
            src: (1, 1),
            dst: (0, 0),
            capture: None,
        });

        let expect = make_board(
            concat!(
                "b..", //
                "..."  //
            ),
            3,
        );

        assert_boardequal!(&c.board, &expect);
    }

    #[tokio::test]
    async fn can_jump() {
        let mut c = Checkers::from(
            concat!(
                "...", //
                ".w.", //
                "..b", //
            ),
            Turn::Black,
            3,
        );

        c.make_step(Step {
            src: (2, 2),
            dst: (0, 0),
            capture: Some((1, 1)),
        });

        let expect = make_board(
            concat!(
                "b..", //
                "...", //
                "...", //
            ),
            3,
        );

        assert_boardequal!(&c.board, &expect);
        assert_eq!(c.turn, Turn::White);

        #[tokio::test]
        async fn can_jump_sequence() {
            let mut c = Checkers::from(
                concat!(
                    "....", //
                    "....", //
                    ".w..", //
                    "....", //
                    "..w.", //
                    "...b", //
                ),
                Turn::Black,
                3,
            );
    
            c.make_step(Step {
                src: (2, 2),
                dst: (0, 0),
                capture: Some((1, 1)),
            });
    
            let expect = make_board(
                concat!(
                    "b..", //
                    "...", //
                    "...", //
                ),
                3,
            );
    
            assert_boardequal!(&c.board, &expect);
            assert_eq!(c.turn, Turn::White);
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
