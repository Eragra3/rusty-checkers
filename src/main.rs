#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::char;
use std::fmt;
use std::io;
use std::io::prelude::*;

// change str to char
const BORDER_TLC: char = '┌';
const BORDER_TRC: char = '┐';
const BORDER_BRC: char = '┘';
const BORDER_BLC: char = '└';

const BORDER_HORIZONTAL: char = '─';
const BORDER_VERTICAL: char = '│';

const NEWLINE: char = '\n';

const BOARD_EMPTY: char = ' ';
const BOARD_WHITE: char = '○';
const BOARD_BLACK: char = '●';
const BOARD_WHITE_KING: char = '♔';
const BOARD_BLACK_KING: char = '♚';

const BOARD_SIZE: usize = 8;

#[derive(Debug, Copy, Clone, PartialEq)]
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

    println!("Note that your terminal may change pawn apperance, here is reference:");
    println!("  White man: {}", BOARD_WHITE);
    println!("  White king: {}", BOARD_WHITE_KING);
    println!("  Black man: {}", BOARD_BLACK);
    println!("  Black king: {}", BOARD_BLACK_KING);
    println!();

    let mut game = Game::new();

    loop {
        game.draw_board();

        print!("Make move: ");
        io::stdout().flush().expect("IO error");
        let mut move_description = String::new();
        match io::stdin().read_line(&mut move_description) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
        let success = game.make_move(&move_description);
        println!("Moved? {}", success);

        if game.state == GameState::Won(Player::White) {
            println!("White won!");
        } else if game.state == GameState::Won(Player::Black) {
            println!("Black won!");
        }
    }
}

#[derive(Debug)]
struct Board {
    height: usize,
    width: usize,
    tiles: Box<[Tile]>,
}

// todo: add two accessors/iterators, one regular and one for rotated board
//       this will eliminate the need of having two boards in Game
impl Board {
    /// Create board with height x width size. You can flip the board to swap players
    fn new(height: usize, width: usize, flip: bool) -> Board {
        // assert that board size is even vertically
        assert_eq!(
            height % 2,
            0,
            "Board vertical size cannot be odd! Got {}",
            height
        );
        let mut tiles = vec![Tile::Empty; height * width];

        let top_tile = if flip { Tile::White } else { Tile::Black };
        let bottom_tile = if flip { Tile::Black } else { Tile::White };

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
                        top_tile
                    } else {
                        bottom_tile
                    };
                }
            }
        }

        Board {
            height,
            width,
            tiles: tiles.into_boxed_slice(),
        }
    }

    pub fn get_tile(&self, Index { x, y }: &Index) -> Tile {
        assert!(*x < self.height, "X coordinate is outside board! Got {}", x);
        assert!(*y < self.width, "Y coordinate is outside board! Got {}", y);

        self.tiles[x + y * self.width]
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn set_tile(&mut self, Index { x, y }: &Index, tile: Tile) {
        assert!(*x < self.height, "X coordinate is outside board! Got {}", x);
        assert!(*y < self.width, "Y coordinate is outside board! Got {}", y);

        self.tiles[x + y * self.width] = tile;
    }

    pub fn get_drawed_board(&self) -> String {
        // todo: notation bars should be created based on board width
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
                let tile = match self.get_tile(&Index::new(x, y)) {
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

#[derive(Debug, PartialEq)]
enum Player {
    White,
    Black,
}

#[derive(Debug, PartialEq)]
enum GameState {
    Turn(Player),
    Won(Player),
}

#[derive(Debug)]
struct Game {
    board: Board,
    turned_board: Board,
    state: GameState,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: Board::new(8, 8, false),
            turned_board: Board::new(8, 8, true),
            state: GameState::Turn(Player::Black),
        }
    }
    ///
    /// Moves are using chess-like algebraic notation, without pawn descriptions.
    /// We use pair of letter + numbers, delimited by single space:
    /// `A6 B5` or `B1 C2`
    ///
    // todo: implement
    pub fn make_move(&mut self, description: &str) -> bool {
        let board_move = match self.parse_move_description(description) {
            Some(m) => m,
            None => return false,
        };

        if !self.check_move(&board_move) {
            return false;
        }

        let pawn = self.board.get_tile(&board_move.source);
        // remove pawn from source
        self.board.set_tile(&board_move.source, Tile::Empty);
        // put pawn in target
        self.board.set_tile(&board_move.target, pawn);

        // change turn
        self.change_turn();

        true
    }

    fn change_turn(&mut self) {
        self.state = if self.state == GameState::Turn(Player::Black) {
            GameState::Turn(Player::White)
        }
        else {
            GameState::Turn(Player::Black)
        }
    }

    fn parse_move_description(&self, description: &str) -> Option<Move> {
        lazy_static! {
            static ref ALGEBRAIC_NOTATION_REGEX: Regex =
                Regex::new("([A-Z])([0-9]+) ([A-Z])([0-9]+)").unwrap();
        }

        if !ALGEBRAIC_NOTATION_REGEX.is_match(description) {
            return None;
        }

        // we can safely unwrap all of below, because regex is matching as per check above
        let captures = ALGEBRAIC_NOTATION_REGEX
            .captures_iter(description)
            .nth(0)
            .unwrap();

        let source_letter: char = captures[1].chars().nth(0).unwrap();
        let source_number: u32 = captures[2].parse().unwrap();
        let target_letter: char = captures[3].chars().nth(0).unwrap();
        let target_number: u32 = captures[4].parse().unwrap();

        // horizontal indeces are created from A-Z letters
        // we can use their char code value and subtract A value
        // vertical indeces are created from 1-based numbers
        // we only have to subtract 1
        let source_horizontal_index = (source_letter as usize) - 65;
        let source_vertical_index = (source_number - 1) as usize;
        let target_horizontal_index = (target_letter as usize) - 65;
        let target_vertical_index = (target_number - 1) as usize;

        Some(Move {
            source: Index::new(source_horizontal_index, source_vertical_index),
            target: Index::new(target_horizontal_index, target_vertical_index),
        })
    }

    // todo: return specific errors instead of boolean
    // todo: simplify calculation by using flipped board and using the same calculation for white player, but on
    //       horizontally flipped board
    pub fn check_move(&self, game_move: &Move) -> bool {

        // we are using regular board for black player and 
        let board = match self.state {
            GameState::Turn(Player::Black) => &self.board,
            GameState::Turn(Player::White) => &self.turned_board,
            GameState::Won(_) => return false,
        };

        // check if source tile is a pawn
        let source_tile = match board.get_tile(&game_move.source) {
            Tile::Empty => return false,
            tile => tile,
        };

        // check if current turn player is the owner
        match self.state {
            GameState::Turn(Player::Black) => {
                if source_tile == Tile::White || source_tile == Tile::WhiteKing {
                    return false;
                }
            }
            GameState::Turn(Player::White) => {
                if source_tile == Tile::Black || source_tile == Tile::BlackKing {
                    return false;
                }
            }
            GameState::Won(_) => return false,
        }

        // check if it's crowned pawn (king)
        if source_tile == Tile::BlackKing || source_tile == Tile::WhiteKing {
            // king move
            panic!("King moves are not implemented!");
        } else {
            // man move

            // check if move is forward
            match self.state {
                GameState::Turn(Player::Black) => {
                    if game_move.source.y >= game_move.target.y {
                        return false;
                    }
                }
                GameState::Turn(Player::White) => {
                    if game_move.source.y <= game_move.target.y {
                        return false;
                    }
                }
                _ => return false,
            }

            // check if it's a simple move or jump
            

            if false {
                // todo simple move
            }
            // else if true {
            //     // todo jump
            // } else {
            //     // invalid move
            //     return false;
        }
        true
    }

    pub fn draw_board(&self) {
        let board = self.board.get_drawed_board();

        println!("{}", board);
    }
}

#[derive(Debug)]
struct Move {
    source: Index,
    target: Index,
}

impl Move {
    pub fn new(source: Index, target: Index) -> Move {
        Move { source, target }
    }
}

#[derive(Debug)]
struct Index {
    x: usize,
    y: usize,
}

impl Index {
    /// Create new index, x is horizontal index, y is vertical. Indeces are 0-based
    pub fn new(x: usize, y: usize) -> Index {
        Index { x, y }
    }
}

trait Indexer2D {
    type Item;

    fn get_item(index: &Index) -> &Self::Item {
        unimplemented!();
    }
}
