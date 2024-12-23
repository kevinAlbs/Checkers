use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Turn {
    Black,
    White,
}

struct Board {
    data: [[char; 8]; 8],
}

impl Board {
    fn get(&self, i: i8, j: i8) -> char {
        assert!(Self::contains(i, j));
        return self.data[i as usize][j as usize];
    }
    fn set(&mut self, i: i8, j: i8, v: char) {
        assert!(Self::contains(i, j));
        self.data[i as usize][j as usize] = v;
    }
    fn contains(i: i8, j: i8) -> bool {
        return i >= 0 && i <= 7 && j >= 0 && j <= 7;
    }
    fn from_strings(s: Vec<&str>) -> Board {
        let mut board = Board {
            data: [['.'; 8]; 8],
        };
        for i in 0..s.len() {
            for j in 0..s[i].len() {
                board.data[i][j] = s[i].chars().nth(j).unwrap();
            }
        }
        return board;
    }
    fn to_string(&self) -> String {
        let mut as_str = String::new();
        for row in self.data {
            for square in row {
                as_str.push(square);
            }
            as_str.push('\n');
        }
        return as_str;
    }
}

#[wasm_bindgen]
struct Checkers {
    turn: Turn,
    board: Board,
    in_step: Option<(i8, i8)>,
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
            board: Board::from_strings(s),
            in_step: None,
        };

        return ch;
    }

    #[wasm_bindgen]
    pub fn at(&self, i: i8, j: i8) -> char {
        if !Board::contains(i, j) {
            return '?';
        }
        return self.board.get(i, j);
    }

    #[wasm_bindgen]
    pub fn get_turn(&self) -> char {
        return match self.turn {
            Turn::Black => 'b',
            Turn::White => 'w',
        };
    }

    fn is_opponent(&self, i: i8, j: i8) -> bool {
        if !Board::contains(i, j) {
            return false;
        }
        let square = self.board.get(i, j);
        return match self.turn {
            Turn::Black => square == 'w' || square == 'W',
            Turn::White => square == 'b' || square == 'B',
        };
    }
    fn is_empty(&self, i: i8, j: i8) -> bool {
        if !Board::contains(i, j) {
            return false;
        }
        return self.board.get(i, j) == '.';
    }

    #[wasm_bindgen]
    pub fn make_step(&mut self, s: Step) {
        let steps = self.get_steps(s.src.0, s.src.1);
        assert!(steps.contains(&s));
        let piece = self.board.get(s.src.0, s.src.1);
        self.board.set(s.src.0, s.src.1, '.');
        assert_eq!(self.board.get(s.dst.0, s.dst.1), '.');
        self.board.set(s.dst.0, s.dst.1, piece);
        if s.capture.is_some() {
            let jump_i = s.capture.unwrap().0;
            let jump_j = s.capture.unwrap().1;
            assert!(self.is_opponent(jump_i, jump_j));
            self.board.set(jump_i, jump_j, '.');
            self.in_step = Some((s.dst.0, s.dst.1));
            if self.get_steps(s.dst.0, s.dst.1).len() == 0 {
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
                    self.board.set(s.dst.0, s.dst.1, 'W');
                }
            }
            'b' => {
                if s.dst.0 == 0 {
                    self.board.set(s.dst.0, s.dst.1, 'B');
                }
            }
            _ => {}
        }
    }

    // Return all steps for a position.
    #[wasm_bindgen]
    pub fn get_steps(&self, i: i8, j: i8) -> Vec<Step> {
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
    fn get_steps_local(&self, i: i8, j: i8) -> Vec<Step> {
        let piece = self.board.get(i, j);

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
                if in_step != (i, j) {
                    // If in a jump sequence, only the piece that made the jump can make the next step.
                    return vec![];
                }
            }
            None => {}
        }

        let mut steps = Vec::<Step>::new();

        if self.in_step.is_none() {
            // Check for ordinary move.
            let mut maybe_step = |src_i: i8, src_j: i8, dst_i: i8, dst_j: i8| {
                if self.is_empty(dst_i, dst_j) {
                    steps.push(Step {
                        src: (src_i, src_j),
                        dst: (dst_i, dst_j),
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

        let mut maybe_jump =
            |src_i: i8, src_j: i8, jump_i: i8, jump_j: i8, dst_i: i8, dst_j: i8| {
                if self.is_opponent(jump_i, jump_j) && self.is_empty(dst_i, dst_j) {
                    steps.push(Step {
                        src: (src_i, src_j),
                        dst: (dst_i, dst_j),
                        capture: Some((jump_i, jump_j)),
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
    src: (i8, i8),
    dst: (i8, i8),
    capture: Option<(i8, i8)>,
}

#[wasm_bindgen]
impl Step {
    #[wasm_bindgen(getter)]
    pub fn src_i(&self) -> i8 {
        return self.src.0;
    }
    #[wasm_bindgen(getter)]
    pub fn src_j(&self) -> i8 {
        return self.src.1;
    }
    #[wasm_bindgen(getter)]
    pub fn dst_i(&self) -> i8 {
        return self.dst.0;
    }
    #[wasm_bindgen(getter)]
    pub fn dst_j(&self) -> i8 {
        return self.dst.1;
    }
}

impl std::fmt::Display for Checkers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..8 {
            for j in 0..8 {
                write!(f, "{}", self.board.get(i, j))?;
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
            let a_str = $a.to_string();
            let b_str = $b.to_string();
            assert_eq!(a_str, b_str, "\n{}!=\n{}", a_str, b_str);
        };
    }

    #[test]
    fn can_display_board() {
        let c = Checkers::new();
        let expect = Board::from_strings(vec![
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

        let expect = Board::from_strings(vec![
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

        let expect = Board::from_strings(vec![
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

        let expect = Board::from_strings(vec![
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

        let expect = Board::from_strings(vec![
            ".B", //
            "..", //
        ]);

        assert_boardequal!(&c.board, &expect);
        assert_eq!(c.turn, Turn::White);
    }
}
