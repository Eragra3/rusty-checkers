use std::fmt;
use std::char;

// change str to char
const BORDER_TLC: char = '┌';
const BORDER_TRC: char = '┐';
const BORDER_BRC: char = '┘';
const BORDER_BLC: char = '└';

const BORDER_HORIZONTAL: char = '─';
const BORDER_VERTICAL: char = '│';

const NEWLINE: char = '\n';

const BOARD_EMPTY: char = ' ';
const BOARD_WHITE: char = 'O';
const BOARD_BLACK: char = 'X';
const BOARD_WHITE_KING: char = 'O';
const BOARD_BLACK_KING: char = 'X';

const BOARD_SIZE: usize = 8;

#[derive(Debug, Copy, Clone)]
enum Tile {
    Empty,
    White,
    Black,
    BlackKing,
    WhiteKing,
}

fn main() {
    // assert that board size is even vertically
    assert_eq!(
        BOARD_SIZE % 2,
        0,
        "Board vertical size cannot be odd! Got {}",
        BOARD_SIZE
    );

    let board = Board::new(BOARD_SIZE, BOARD_SIZE);

    println!("{}", board);
}

#[derive(Debug)]
struct Board {
    height: usize,
    width: usize,
    tiles: Box<[Tile]>,
}

impl Board {
    fn new(height: usize, width: usize) -> Board {
        // assert that board size is even vertically
        assert_eq!(
            height % 2,
            0,
            "Board vertical size cannot be odd! Got {}",
            height
        );
        let mut tiles = vec![Tile::Empty; height * width];

        for y in 0..height {
            for x in 0..width {
                // two empty lines in the middle
                if y == height / 2 || y == height / 2 - 1 {
                    continue;
                }

                /* classic setup, alternately:
                 empty -> pawn  -> empty -> pawn
                 pawn  -> empty -> pawn  -> empty
                 pattern from top to bottom
                */
                if (x + y) % 2 == 1 {
                    tiles[x + y * width] = if y < height / 2 {
                        Tile::White
                    } else {
                        Tile::Black
                    };
                }
            }
        }

        Board {
            height: height,
            width: width,
            tiles: tiles.into_boxed_slice(),
        }
    }

    pub fn get_tile(&self, (x, y): (usize, usize)) -> Tile {
        assert!(x < self.height, "X coordinate is outside board! Got {}", x);
        assert!(y < self.width, "Y coordinate is outside board! Got {}", y);

        self.tiles[x + y * self.width]
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn set_tile(&mut self, (x, y): (usize, usize), tile: Tile) {
        assert!(x < self.height, "X coordinate is outside board! Got {}", x);
        assert!(y < self.width, "Y coordinate is outside board! Got {}", y);
        self.tiles[x + y * self.width] = tile;
    }

    pub fn get_drawed_board(&self) -> String {
        let horizontal_notation = format!("  ABCDEFGH  {}", NEWLINE);

        let mut top_border = String::new();
        top_border.push(' ');
        top_border.push(BORDER_TLC);
        for _ in 0..self.width() {
            top_border.push(BORDER_HORIZONTAL);
        }
        top_border.push(BORDER_TRC);
        top_border.push(' ');
        top_border.push(NEWLINE);

        let mut bottom_border = String::new();
        bottom_border.push(' ');
        bottom_border.push(BORDER_BLC);
        for _ in 0..self.width() {
            bottom_border.push(BORDER_HORIZONTAL);
        }
        bottom_border.push(BORDER_BRC);
        bottom_border.push(' ');
        bottom_border.push(NEWLINE);


        let mut printed_board = String::new();

        printed_board.push_str(&horizontal_notation);
        printed_board.push_str(&top_border);
        for y in 0..self.height {
            let mut middle_row = String::new();

            middle_row.push(char::from_digit((y + 1) as u32, 10).unwrap());
            middle_row.push(BORDER_VERTICAL);
            for x in 0..self.width {
                let tile = match self.get_tile((x, y)) {
                    Tile::Empty => BOARD_EMPTY,
                    Tile::White => BOARD_WHITE,
                    Tile::Black => BOARD_BLACK,
                    Tile::WhiteKing => BOARD_WHITE_KING,
                    Tile::BlackKing => BOARD_BLACK_KING,
                };

                middle_row.push(tile);
            }
            middle_row.push(BORDER_VERTICAL);
            middle_row.push(char::from_digit((y + 1) as u32, 10).unwrap());
            middle_row.push(NEWLINE);
            printed_board.push_str(&middle_row);
        }
        printed_board.push_str(&bottom_border);
        printed_board.push_str(&horizontal_notation);

        printed_board
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_drawed_board())
    }
}
