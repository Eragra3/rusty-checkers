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

        println!("{:?}", game.state);
        print!("Make move: ");
        io::stdout().flush().expect("IO error");
        let mut move_description = String::new();
        match io::stdin().read_line(&mut move_description) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
        let success = game.make_move(&move_description);
        println!("Moved? {:?}", success);

        if game.state == GameState::Won(Player::White) {
            println!("White won!");
        } else if game.state == GameState::Won(Player::Black) {
            println!("Black won!");
        }
    }
}

/// Checkers board representation.
///
/// `get_tile_white` and `set_tile_white` give access to tiles from white player perspective.
/// (0,0) point is in top-left corner:
/// | (x, y)      |        |     |                 |
/// | ----------- | ------ | --- | --------------- |
/// | (0, 0)      | (1, 0) | ... | (width, 0)      |
/// | (0, 1)      | (1, 1) |     |                 |
/// | ...         |        | ... |                 |
/// | (0, height) |        |     | (width, height) |
///
/// `get_tile_black` and `set_tile_black` give access to tiles from black player perspective.
/// Reversed indexer has (0,0) point in bottom-right corner instead of top-left:
/// | (x, y)          |     |        |             |
/// | --------------- | --- | ------ | ----------- |
/// | (width, height) |     |        | (0, height) |
/// |                 | ... |        |             |
/// |                 |     | (1, 1) | (0, 1)      |
/// | (width, 0)      | ... | (1, 0) | (0, 0)      |
///
/// Methods `get_tile` and `set_tile` will pick correct board orientation based on player
///
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

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get_tile(&self, index: Index, player: Player) -> Tile {
        match player {
            Player::White => self.get_tile_white(index),
            Player::Black => self.get_tile_black(index),
        }
    }

    pub fn set_tile(&mut self, index: Index, tile: Tile, player: Player) {
        match player {
            Player::White => self.set_tile_white(index, tile),
            Player::Black => self.set_tile_black(index, tile),
        }
    }

    /// Get title looking at board from white player perspective
    pub fn get_tile_white(&self, Index { x, y }: Index) -> Tile {
        assert!(x < self.height, "X coordinate is outside board! Got {}", x);
        assert!(y < self.width, "Y coordinate is outside board! Got {}", y);

        self.tiles[x + y * self.width]
    }

    /// Set title looking at board from white player perspective
    pub fn set_tile_white(&mut self, Index { x, y }: Index, tile: Tile) {
        assert!(x < self.height, "X coordinate is outside board! Got {}", x);
        assert!(y < self.width, "Y coordinate is outside board! Got {}", y);

        self.tiles[x + y * self.width] = tile;
    }

    /// Get title looking at board from black player perspective
    pub fn get_tile_black(&self, index: Index) -> Tile {
        let reversed_index = Index::new(self.width - index.x - 1, self.height - index.y - 1);
        println!("Index: {:?}, Reversed index: {:?}", index, reversed_index);
        self.get_tile_white(reversed_index)
    }

    /// Set title looking at board from black player perspective
    pub fn set_tile_black(&mut self, index: Index, tile: Tile) {
        let reversed_index = Index::new(self.width - index.x - 1, self.height - index.y - 1);
        println!("Index: {:?}, Reversed index: {:?}", index, reversed_index);
        self.set_tile_white(reversed_index, tile)
    }

    pub fn get_drawed_board(&self) -> String {
        let player = Player::White;

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
                let tile = match self.get_tile(Index::new(x, y), player) {
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

#[derive(Debug, PartialEq, Copy, Clone)]
enum Player {
    White,
    Black,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum GameState {
    Turn(Player),
    Won(Player),
}

#[derive(Debug)]
struct Game {
    board: Board,
    state: GameState,
}

impl Game {
    pub fn new() -> Game {
        Game {
            board: Board::new(8, 8, false),
            state: GameState::Turn(Player::Black),
        }
    }
    ///
    /// Moves are using chess-like algebraic notation, without pawn descriptions.
    /// We use pair of letter + numbers, delimited by single space:
    /// `A6 B5` or `B1 C2`
    ///
    // todo: implement
    pub fn make_move(&mut self, description: &str) -> Result<(), &str> {
        if let GameState::Won(_) = self.state {
            return Err("You can't make a move, the game has already ended");
        };

        let board_move = match self.parse_move_description(description) {
            Some(m) => m,
            None => return Err("Move description is not well formed"),
        };

        if let err @ Err(_) = self.check_move(board_move) {
            return err;
        }

        let pawn = self.board.get_tile_white(board_move.source);
        // remove pawn from source
        self.board.set_tile_white(board_move.source, Tile::Empty);
        // put pawn in target
        self.board.set_tile_white(board_move.target, pawn);

        // change turn
        self.change_turn();

        Ok(())
    }

    fn change_turn(&mut self) {
        self.state = if self.state == GameState::Turn(Player::Black) {
            GameState::Turn(Player::White)
        } else {
            GameState::Turn(Player::Black)
        }
    }

    /// Parses move notation to an Index:
    /// Ex. `A6 B5` or `B1 C2`.
    ///
    /// The move is indexed from white player perspective.
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

        let game_move = Move::new(
            Index::new(source_horizontal_index, source_vertical_index),
            Index::new(target_horizontal_index, target_vertical_index),
        );

        Some(game_move)
    }

    pub fn check_move<'a>(&self, game_move_white: Move) -> Result<(), &'a str> {
        let player = match self.state {
            GameState::Turn(Player::Black) => Player::Black,
            GameState::Turn(Player::White) => Player::White,
            GameState::Won(_) => return Err("The game is already finished"),
        };

        let game_move = match player {
            Player::Black => self.reverse_move(&game_move_white),
            Player::White => game_move_white,
        };

        println!("Move: {:?}", game_move);

        // check if source tile is a pawn
        let source_tile = match self.board.get_tile(game_move.source, player) {
            Tile::Empty => return Err("Source tile is empty"),
            tile => tile,
        };
        // check if target tile is empty
        match self.board.get_tile(game_move.target, player) {
            Tile::Empty => (),
            _ => return Err("Target tile is not empty"),
        };

        // check if current turn player is the owner
        match player {
            Player::Black => {
                if source_tile == Tile::White || source_tile == Tile::WhiteKing {
                    return Err("The pawn belongs to other player (white)");
                }
            }
            Player::White => {
                if source_tile == Tile::Black || source_tile == Tile::BlackKing {
                    return Err("The pawn belongs to other player (black)");
                }
            }
        }

        // check if it's crowned pawn (king)
        if source_tile == Tile::BlackKing || source_tile == Tile::WhiteKing {
            // king move
            panic!("King moves are not implemented!");
        } else {
            // man move

            // check if move is forward
            if game_move.source.y <= game_move.target.y {
                return Err("The move has to be forward");
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
        Ok(())
    }

    pub fn draw_board(&self) {
        let board = self.board.get_drawed_board();

        println!("{}", board);
    }

    fn reverse_move(&self, game_move: &Move) -> Move {
        let source = self.reverse_index(&game_move.source);
        let target = self.reverse_index(&game_move.target);
        Move { source, target }
    }

    fn reverse_index(&self, index: &Index) -> Index {
        let x = self.board.width() - index.x - 1;
        let y = self.board.height() - index.y - 1;
        Index { x, y }
    }
}

//TODO: move `player` to this structure and remove it as method parameter
#[derive(Debug, Clone, Copy)]
struct Move {
    source: Index,
    target: Index,
}

impl Move {
    pub fn new(source: Index, target: Index) -> Move {
        Move { source, target }
    }
}

#[derive(Debug, Copy, Clone)]
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
