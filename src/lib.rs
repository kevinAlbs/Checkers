use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Turn {
    Black,
    White,
}

#[wasm_bindgen]
struct Checkers {
    turn: Turn,
    board: [[char; 8]; 8],
    in_step: Option<(u8, u8)>,
}

fn make_board(s: Vec<&str>) -> [[char; 8]; 8] {
    let mut board = [['.'; 8]; 8];

    for i in 0..s.len() {
        for j in 0..s[i].len() {
            board[i][j] = s[i].chars().nth(j).unwrap();
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

#[wasm_bindgen]
impl Checkers {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Checkers {
        return Checkers::from(
            vec![
                ".w.w.w.w", //
                "w.w.w.w.", //
                ".w.w.w.w", //
                "........", //
                "........", //
                "b.b.b.b.", //
                ".b.b.b.b", //
                "b.b.b.b.", //
            ],
            Turn::Black,
        );
    }

    fn from(s: Vec<&str>, t: Turn) -> Checkers {
        let ch = Checkers {
            turn: t,
            board: make_board(s),
            in_step: None,
        };

        return ch;
    }

    #[wasm_bindgen]
    pub fn at(&self, i: isize, j: isize) -> char {
        if i < 0 || j < 0 || i > 7 || j > 7 {
            return '?';
        }
        return self.board[i as usize][j as usize];
    }

    #[wasm_bindgen]
    pub fn get_turn(&self) -> char {
        return match self.turn {
            Turn::Black => 'b',
            Turn::White => 'w',
        };
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

    #[wasm_bindgen]
    pub fn make_step(&mut self, s: Step) {
        let steps = self.get_steps(s.src.0 as usize, s.src.1 as usize);
        assert!(steps.contains(&s));
        let piece = self.board[s.src.0 as usize][s.src.1 as usize];
        self.board[s.src.0 as usize][s.src.1 as usize] = '.';
        assert_eq!(self.board[s.dst.0 as usize][s.dst.1 as usize], '.');
        self.board[s.dst.0 as usize][s.dst.1 as usize] = piece;
        if s.capture.is_some() {
            let jump_i = s.capture.unwrap().0;
            let jump_j = s.capture.unwrap().1;
            assert!(Self::is_opponent(
                self.turn,
                &self.board,
                jump_i as isize,
                jump_j as isize
            ));
            self.board[jump_i as usize][jump_j as usize] = '.';
            self.in_step = Some((s.dst.0, s.dst.1));
            if self.get_steps(s.dst.0 as usize, s.dst.1 as usize).len() == 0 {
                // No further jumps are possible. Switch turn.
                self.in_step = None;
                self.turn = match self.turn {
                    Turn::White => Turn::Black,
                    Turn::Black => Turn::White,
                }
            }
        } else {
            // Regular move. Switch turn.
            self.turn = match self.turn {
                Turn::White => Turn::Black,
                Turn::Black => Turn::White,
            }
        }
        // Check if piece is promoted to king.
        match piece {
            'w' => {
                if s.dst.0 == 7 {
                    self.board[s.dst.0 as usize][s.dst.1 as usize] = 'W';
                }
            }
            'b' => {
                if s.dst.0 == 0 {
                    self.board[s.dst.0 as usize][s.dst.1 as usize] = 'B';
                }
            }
            _ => {}
        }
    }

    // Return all steps for a position.
    #[wasm_bindgen]
    pub fn get_steps(&self, i: usize, j: usize) -> Vec<Step> {
        // Check if any piece can capture.
        let mut other_has_capture = false;
        'outer: for oi in 0..8 {
            for oj in 0..8 {
                if oi == i && oj == j {
                    // Skip self.
                    continue;
                }
                let steps = self.get_steps_local(oi, oj);
                other_has_capture = steps.len() > 0 && steps[0].capture.is_some();
                if other_has_capture {
                    break 'outer;
                }
            }
        }

        let steps = self.get_steps_local(i, j);
        let has_capture = steps.len() > 0 && steps[0].capture.is_some();
        if !has_capture && other_has_capture {
            return vec![];
        }
        return steps;
    }

    // Does not account other pieces capturing.
    fn get_steps_local(&self, i: usize, j: usize) -> Vec<Step> {
        let piece = self.board[i][j];

        // Ensure piece matches turn.
        match self.turn {
            Turn::Black => {
                if piece != 'b' && piece != 'B' {
                    return vec![];
                }
            }
            Turn::White => {
                if piece != 'w' && piece != 'W' {
                    return vec![];
                }
            }
        }

        match self.in_step {
            Some(in_step) => {
                if in_step != (i as u8, j as u8) {
                    // If in a jump sequence, only the piece that made the jump can make the next step.
                    return vec![];
                }
            }
            None => {}
        }

        let mut steps = Vec::<Step>::new();
        let i: isize = i as isize;
        let j: isize = j as isize;

        if self.in_step.is_none() {
            // Check for ordinary move.
            let mut maybe_step = |src_i: isize, src_j: isize, dst_i: isize, dst_j: isize| {
                if Self::is_empty(&self.board, dst_i, dst_j) {
                    steps.push(Step {
                        src: (src_i as u8, src_j as u8),
                        dst: (dst_i as u8, dst_j as u8),
                        capture: None,
                    });
                }
            };

            match piece {
                'b' => {
                    // Check NW and NE.
                    maybe_step(i, j, i - 1, j - 1);
                    maybe_step(i, j, i - 1, j + 1);
                }
                'w' => {
                    // Check SW and SE.
                    maybe_step(i, j, i + 1, j - 1);
                    maybe_step(i, j, i + 1, j + 1);
                }
                'B' | 'W' => {
                    // Check NW, NE, SW, SE.
                    maybe_step(i, j, i - 1, j - 1);
                    maybe_step(i, j, i - 1, j + 1);
                    maybe_step(i, j, i + 1, j - 1);
                    maybe_step(i, j, i + 1, j + 1);
                }
                _ => panic!("Unexpected"),
            }
        }

        let mut maybe_jump = |src_i: isize,
                              src_j: isize,
                              jump_i: isize,
                              jump_j: isize,
                              dst_i: isize,
                              dst_j: isize| {
            if Self::is_opponent(self.turn, &self.board, jump_i, jump_j)
                && Self::is_empty(&self.board, dst_i, dst_j)
            {
                steps.push(Step {
                    src: (src_i as u8, src_j as u8),
                    dst: (dst_i as u8, dst_j as u8),
                    capture: Some((jump_i as u8, jump_j as u8)),
                });
            }
        };

        match piece {
            'b' => {
                // Check NW and NE.
                maybe_jump(i, j, i - 1, j - 1, i - 2, j - 2);
                maybe_jump(i, j, i - 1, j + 1, i - 2, j + 2);
            }
            'w' => {
                // Check SW and SE.
                maybe_jump(i, j, i + 1, j - 1, i + 2, j - 2);
                maybe_jump(i, j, i + 1, j + 1, i + 2, j + 2);
            }
            'B' | 'W' => {
                // Check NW, NE, SW, SE.
                maybe_jump(i, j, i - 1, j - 1, i - 2, j - 2);
                maybe_jump(i, j, i - 1, j + 1, i - 2, j + 2);
                maybe_jump(i, j, i + 1, j - 1, i + 2, j - 2);
                maybe_jump(i, j, i + 1, j + 1, i + 2, j + 2);
            }
            _ => panic!("Unexpected"),
        }

        // If there are any moves with a capture, remove non-capture moves.
        let mut has_capture = false;
        for s in &steps {
            if s.capture.is_some() {
                has_capture = true;
                break;
            }
        }

        if has_capture {
            // Keep only moves that have a capture.
            steps.retain(|el| {
                return el.capture.is_some();
            });
        }

        return steps;
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Clone, Copy)]
struct Step {
    src: (u8, u8),
    dst: (u8, u8),
    capture: Option<(u8, u8)>,
}

#[wasm_bindgen]
impl Step {
    #[wasm_bindgen(getter)]
    pub fn src_i(&self) -> u8 {
        return self.src.0;
    }
    #[wasm_bindgen(getter)]
    pub fn src_j(&self) -> u8 {
        return self.src.1;
    }
    #[wasm_bindgen(getter)]
    pub fn dst_i(&self) -> u8 {
        return self.dst.0;
    }
    #[wasm_bindgen(getter)]
    pub fn dst_j(&self) -> u8 {
        return self.dst.1;
    }
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

    macro_rules! assert_boardequal {
        ($a:expr, $b:expr) => {
            let a_str = board_to_string($a);
            let b_str = board_to_string($b);
            assert_eq!(a_str, b_str, "\n{}!=\n{}", a_str, b_str);
        };
    }

    #[test]
    fn can_display_board() {
        let c = Checkers::new();
        let expect = make_board(vec![
            ".w.w.w.w", //
            "w.w.w.w.", //
            ".w.w.w.w", //
            "........", //
            "........", //
            "b.b.b.b.", //
            ".b.b.b.b", //
            "b.b.b.b.", //
        ]);
        assert_boardequal!(&c.board, &expect);
    }

    #[test]
    fn finds_moves() {
        let c = Checkers::from(
            vec![
                ".w.", //
                "...", //
            ],
            Turn::White,
        );

        let steps = c.get_steps(0, 1);
        assert_eq!(
            steps,
            vec![
                Step {
                    src: (0, 1),
                    dst: (1, 0),
                    capture: None
                },
                Step {
                    src: (0, 1),
                    dst: (1, 2),
                    capture: None
                }
            ]
        );
    }

    #[test]
    fn determines_capture() {
        let c = Checkers::from(
            vec![
                ".w.", //
                "..b", //
            ],
            Turn::White,
        );

        let steps = c.get_steps(0, 1);
        assert_eq!(
            steps,
            vec![Step {
                src: (0, 1),
                dst: (2, 3),
                capture: Some((1, 2))
            }]
        );
    }

    #[test]
    fn requires_capture() {
        let c = Checkers::from(
            vec![
                ".w.", //
                "..b", //
            ],
            Turn::White,
        );

        let steps = c.get_steps(0, 1);
        assert_eq!(
            steps,
            vec![Step {
                src: (0, 1),
                dst: (2, 3),
                capture: Some((1, 2))
            }]
        );
    }

    #[test]
    fn requires_capture_for_other_piece() {
        let c = Checkers::from(
            vec![
                ".w...w", //
                "..b...", //
            ],
            Turn::White,
        );

        let steps = c.get_steps(0, 5);
        assert_eq!(steps, vec![]);
    }

    #[test]
    fn requires_jump_sequence_to_continue() {
        let mut c = Checkers::from(
            vec![
                ".w....", //
                "..b.b.", //
                ".....w", //
                "..b.b.", //
            ],
            Turn::White,
        );

        c.make_step(Step {
            src: (0, 1),
            dst: (2, 3),
            capture: Some((1, 2)),
        });

        // Still in sequence. Expect other white piece cannot jump.
        let steps = c.get_steps(2, 5);
        assert_eq!(steps, vec![]);
    }

    #[test]
    fn determines_capture_sequence() {
        let c = Checkers::from(
            vec![
                ".w..", //
                "..b.", //
                "....", //
                "..b.", //
            ],
            Turn::White,
        );

        let steps = c.get_steps(0, 1);
        assert_eq!(
            steps,
            vec![Step {
                src: (0, 1),
                dst: (2, 3),
                capture: Some((1, 2))
            }]
        );
    }

    #[test]
    fn can_step() {
        let mut c = Checkers::from(
            vec![
                ".w", //
                "..", //
            ],
            Turn::White,
        );

        c.make_step(Step {
            src: (0, 1),
            dst: (1, 0),
            capture: None,
        });

        let expect = make_board(vec![
            "..", //
            "w.", //
        ]);

        assert_boardequal!(&c.board, &expect);
    }

    #[test]
    fn can_jump() {
        let mut c = Checkers::from(
            vec![
                ".w..", //
                "..b.", //
                "....", //
            ],
            Turn::White,
        );

        c.make_step(Step {
            src: (0, 1),
            dst: (2, 3),
            capture: Some((1, 2)),
        });

        let expect = make_board(vec![
            "....", //
            "....", //
            "...w", //
        ]);

        let got = c.get_steps(2, 3);
        assert_eq!(got, vec![]);
        assert_boardequal!(&c.board, &expect);
        assert_eq!(c.turn, Turn::Black);
    }

    #[test]
    fn checks_jump_destination_is_empty() {
        let mut c = Checkers::from(
            vec![
                ".w..", //
                "..b.", //
                "...b", //
            ],
            Turn::White,
        );

        let got = c.get_steps(0, 1);
        let expect = vec![Step {
            src: (0, 1),
            dst: (1, 0),
            capture: None,
        }];

        assert_eq!(got, expect);
    }

    #[test]
    fn can_jump_sequence() {
        let mut c = Checkers::from(
            vec![
                ".w..", //
                "..b.", //
                "....", //
                "..b.", //
                "....", //
            ],
            Turn::White,
        );

        c.make_step(Step {
            src: (0, 1),
            dst: (2, 3),
            capture: Some((1, 2)),
        });

        let expect = make_board(vec![
            "....", //
            "....", //
            "...w", //
            "..b.", //
            "....", //
        ]);

        assert_boardequal!(&c.board, &expect);
        assert_eq!(c.turn, Turn::White);
    }

    #[test]
    fn can_king() {
        let mut c = Checkers::from(
            vec![
                "..", //
                "b.", //
            ],
            Turn::Black,
        );

        c.make_step(Step {
            src: (1, 0),
            dst: (0, 1),
            capture: None,
        });

        let expect = make_board(vec![
            ".B", //
            "..", //
        ]);

        assert_boardequal!(&c.board, &expect);
        assert_eq!(c.turn, Turn::White);
    }
}
