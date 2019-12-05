#[macro_use]
extern crate lazy_static;

use colored::*;
use pad::{Alignment, PadStr};
use regex::Regex;
use std::char;
use std::convert::TryFrom;
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
const BOARD_MAN: char = '●';
const BOARD_KING: char = '○';

const INTERNAL_ERROR_MESSAGE: &str = "Internal error, shouldn't get here. Oooops...";

fn get_enemy(player: Player) -> Player {
    match player {
        Player::White => Player::Black,
        Player::Black => Player::White,
    }
}

fn get_tile_owner(tile: Tile) -> Option<Player> {
    match tile {
        Tile::Black | Tile::BlackKing => Some(Player::Black),
        Tile::White | Tile::WhiteKing => Some(Player::White),
        Tile::Empty => None,
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Tile {
    Empty,
    White,
    Black,
    BlackKing,
    WhiteKing,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    NW,
    NE,
    SE,
    SW,
}

fn main() {
    let mut game = Game::new();

    game.draw_info();

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
        move_description = move_description.to_uppercase();
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

impl Board {
    /// Create board with height x width size.
    fn new(height: usize, width: usize) -> Board {
        // assert that board size is even vertically
        assert_eq!(
            height % 2,
            0,
            "Board vertical size cannot be odd! Got {}",
            height
        );
        let mut tiles = vec![Tile::Empty; height * width];

        let top_tile = Tile::Black;
        let bottom_tile = Tile::White;

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

    pub fn get_tile_owner_at(&self, index: Index) -> Result<Option<Player>, &str> {
        let tile = &self.get_tile(index);

        match tile {
            Ok(Tile::White) | Ok(Tile::WhiteKing) => Ok(Some(Player::White)),
            Ok(Tile::Black) | Ok(Tile::BlackKing) => Ok(Some(Player::Black)),
            Ok(Tile::Empty) => Ok(None),
            Err(msg) => Err(msg),
        }
    }

    pub fn get_tile<'a>(&self, index: Index) -> Result<Tile, &'a str> {
        match index.orientation {
            Player::White => self.get_tile_white(index),
            Player::Black => self.get_tile_black(index),
        }
    }

    pub fn set_tile(&mut self, index: Index, tile: Tile) -> Result<(), &str> {
        match index.orientation {
            Player::White => self.set_tile_white(index, tile),
            Player::Black => self.set_tile_black(index, tile),
        }
    }

    /// Get title looking at board from white player perspective
    fn get_tile_white<'a>(&self, index: Index) -> Result<Tile, &'a str> {
        assert!(
            index.orientation == Player::White,
            "`get_tile_white` called with black player Index"
        );

        if !self.validate_index(index) {
            println!("Index outside of board");
            return Err("Index outside of board");
        }

        Ok(self.tiles[index.x + index.y * self.width])
    }

    /// Set title looking at board from white player perspective
    fn set_tile_white(&mut self, index: Index, tile: Tile) -> Result<(), &str> {
        assert!(
            index.orientation == Player::White,
            "`set_tile_white` called with black player Index"
        );

        if !self.validate_index(index) {
            println!("Index outside of board");
            return Err("Index outside of board");
        }

        self.tiles[index.x + index.y * self.width] = tile;

        Ok(())
    }

    /// Get title looking at board from black player perspective
    fn get_tile_black<'a>(&self, index: Index) -> Result<Tile, &'a str> {
        assert!(
            index.orientation == Player::Black,
            "`set_tile_black` called with white player Index"
        );

        if !self.validate_index(index) {
            println!("Index outside of board");
            return Err("Index outside of board");
        }

        let reversed_index = self.reverse_index(&index);
        println!("Index: {:?}, Reversed index: {:?}", index, reversed_index);

        self.get_tile_white(reversed_index)
    }

    /// Set title looking at board from black player perspective
    fn set_tile_black(&mut self, index: Index, tile: Tile) -> Result<(), &str> {
        assert!(
            index.orientation == Player::Black,
            "`set_tile_black` called with white player Index"
        );

        if !self.validate_index(index) {
            println!("Index outside of board");
            return Err("Index outside of board");
        }

        let reversed_index = self.reverse_index(&index);
        println!("Index: {:?}, Reversed index: {:?}", index, reversed_index);

        self.set_tile_white(reversed_index, tile)
    }

    fn validate_index(&self, index: Index) -> bool {
        index.x < self.width && index.y < self.height
    }

    fn reverse_index(&self, index: &Index) -> Index {
        let x = self.width() - index.x - 1;
        let y = self.height() - index.y - 1;

        Index::new(x, y, get_enemy(index.orientation))
    }

    // TODO: add some kind of theme support, the board still looks meh in some terminals
    pub fn get_drawed_board(&self) -> String {
        // number of digits in vertical numeric notation
        let vertical_index_digits = self.height.to_string().len();
        // padding equal to vertical numeric notation width
        let notation_padding = String::new().pad_to_width(vertical_index_digits);

        // horizontal character notation
        let horizontal_notation = (0..self.width)
            .map(|i| char::from_u32((65 + i) as u32).expect("Unsupported width!"))
            .collect::<String>();

        let horizontal_notation_line = format!(
            "{} {} {}",
            notation_padding, horizontal_notation, notation_padding
        );

        let top_border = format!(
            "{}{}{}",
            BORDER_TLC,
            (0..self.width())
                .map(|_| BORDER_HORIZONTAL)
                .collect::<String>(),
            BORDER_TRC,
        )
        .blue()
        .on_white()
        .to_string();
        let top_border_line = format!("{}{}{}", notation_padding, top_border, notation_padding);

        let bottom_border = format!(
            "{}{}{}",
            BORDER_BLC,
            (0..self.width())
                .map(|_| BORDER_HORIZONTAL)
                .collect::<String>(),
            BORDER_BRC,
        )
        .blue()
        .on_white()
        .to_string();
        let bottom_border_line =
            format!("{}{}{}", notation_padding, bottom_border, notation_padding);

        let mut middle_rows: Vec<String> = Vec::new();
        for y in 0..self.height {
            let mut middle_row = String::new();

            let vertical_index = (y + 1)
                .to_string()
                .pad_to_width_with_alignment(vertical_index_digits, Alignment::Left);

            middle_row.push_str(vertical_index.as_str());
            middle_row.push_str(&BORDER_VERTICAL.to_string().blue().on_white().to_string());

            let mut tile_row = String::new();
            for x in 0..self.width {
                let tile = match self.get_tile(Index::new(x, y, Player::White)) {
                    Ok(Tile::Empty) => self.get_empty_space(),
                    Ok(Tile::White) => self.get_white_man(),
                    Ok(Tile::Black) => self.get_black_man(),
                    Ok(Tile::WhiteKing) => self.get_white_king(),
                    Ok(Tile::BlackKing) => self.get_black_king(),
                    Err(msg) => panic!(msg),
                };
                tile_row.push_str(&tile);
            }
            middle_row.push_str(&tile_row.on_blue().to_string());

            middle_row.push_str(&BORDER_VERTICAL.to_string().blue().on_white().to_string());
            middle_row.push_str(&vertical_index);
            middle_row.push(NEWLINE);
            middle_rows.push(middle_row);
        }

        [
            horizontal_notation_line.clone(),
            self.get_newline(),
            top_border_line,
            self.get_newline(),
            middle_rows.concat(),
            bottom_border_line,
            self.get_newline(),
            horizontal_notation_line,
            self.get_newline(),
        ]
        .concat()
    }

    pub fn draw_info(&self) {
        println!("Empty tile: {}", self.get_empty_space().on_blue());
        println!("White man:  {}", self.get_white_man().on_blue());
        println!("White king: {}", self.get_white_king().on_blue());
        println!("Black man:  {}", self.get_black_man().on_blue());
        println!("Black king: {}", self.get_black_king().on_blue());
    }

    fn get_white_man(&self) -> String {
        BOARD_MAN.to_string().white().to_string()
    }

    fn get_black_man(&self) -> String {
        BOARD_MAN.to_string().black().to_string()
    }

    fn get_white_king(&self) -> String {
        BOARD_KING.to_string().white().to_string()
    }

    fn get_black_king(&self) -> String {
        BOARD_KING.to_string().black().to_string()
    }

    fn get_empty_space(&self) -> String {
        BOARD_EMPTY.to_string()
    }

    fn get_newline(&self) -> String {
        NEWLINE.to_string()
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
            board: Board::new(10, 10),
            state: GameState::Turn(Player::White),
        }
    }
    ///
    /// Moves are using chess-like algebraic notation, without pawn descriptions.
    /// We use pair of letter + numbers, delimited by single space:
    /// `A6 B5` or `B1 C2`
    ///
    // todo: implement
    pub fn make_move(&mut self, description: &str) -> Result<(), &str> {
        // Check if the game is still in progress
        if let GameState::Won(_) = self.state {
            return Err("You can't make a move, the game has already ended");
        };

        // Try parsing move description
        let board_move = match self.parse_move_description(description) {
            Ok(m) => m,
            Err(msg) => return Err(msg),
        };

        // Check if the move is valid
        let game_move = match self.check_move(board_move) {
            Ok(game_move) => game_move,
            Err(msg) => return Err(msg),
        };

        // Get source pawn
        let pawn = match self.board.get_tile(game_move.source()) {
            Ok(tile) => tile,
            Err(msg) => return Err(msg),
        };

        // Remove pawn from source
        self.board
            .set_tile(board_move.source, Tile::Empty)
            .expect(INTERNAL_ERROR_MESSAGE);
        // Put pawn in target
        self.board
            .set_tile(board_move.target, pawn)
            .expect(INTERNAL_ERROR_MESSAGE);

        // Remove captured pawns
        match game_move.move_type {
            MoveType::Move(_) | MoveType::KingMove(_) => (),
            MoveType::Capture { captured_index, .. } => self
                .board
                .set_tile(captured_index, Tile::Empty)
                .expect(INTERNAL_ERROR_MESSAGE),
            _ => unimplemented!(),
        }

        // change turn
        self.change_turn();

        Ok(())
    }

    fn change_turn(&mut self) {
        match self.state {
            GameState::Won(_) => panic!("The game has already ended!"),
            GameState::Turn(player) => self.state = GameState::Turn(get_enemy(player)),
        }
    }

    /// Parses move notation to an Index:
    /// Ex. `A6 B5` or `B1 C2`.
    ///
    /// The move is indexed from white player perspective.
    fn parse_move_description<'a>(&self, description: &str) -> Result<Move, &'a str> {
        lazy_static! {
            static ref ALGEBRAIC_NOTATION_REGEX: Regex =
                Regex::new("([A-Z])([0-9]+) ([A-Z])([0-9]+)").unwrap();
        }

        if !ALGEBRAIC_NOTATION_REGEX.is_match(description) {
            return Err("Move does not match required notation");
        }

        // we can safely unwrap all of below, because regex is matching as per check above
        let captures = ALGEBRAIC_NOTATION_REGEX
            .captures_iter(description)
            .nth(0)
            .unwrap();

        let source_letter: char = captures[1].chars().nth(0).unwrap();
        let source_number: i32 = captures[2].parse().unwrap();
        let target_letter: char = captures[3].chars().nth(0).unwrap();
        let target_number: i32 = captures[4].parse().unwrap();

        // horizontal indeces are created from A-Z letters
        // we can use their char code value and subtract A value
        // vertical indeces are created from 1-based numbers
        // we only have to subtract 1
        // TODO: try to refactor this code
        let source_horizontal_index = match usize::try_from((source_letter as i32) - 65) {
            Ok(number) => {
                if number < self.board.height() {
                    number
                } else {
                    return Err("Horizontal source index is outside the board");
                }
            }
            Err(_) => return Err("Horizontal source index is outside the board"),
        };
        let source_vertical_index = match usize::try_from(source_number - 1) {
            Ok(number) => {
                if number < self.board.height() {
                    number
                } else {
                    return Err("Vertical source index is outside the board");
                }
            }
            Err(_) => return Err("Vertical source index is outside the board"),
        };
        let target_horizontal_index = match usize::try_from((target_letter as i32) - 65) {
            Ok(number) => {
                if number < self.board.height() {
                    number
                } else {
                    return Err("Horizontal target index is outside the board");
                }
            }
            Err(_) => return Err("Horizontal target index is outside the board"),
        };
        let target_vertical_index = match usize::try_from(target_number - 1) {
            Ok(number) => {
                if number < self.board.height() {
                    number
                } else {
                    return Err("Vertical target index is outside the board");
                }
            }
            Err(_) => return Err("Vertical target index is outside the board"),
        };

        let game_move = Move::new(
            Index::new(
                source_horizontal_index,
                source_vertical_index,
                Player::White,
            ),
            Index::new(
                target_horizontal_index,
                target_vertical_index,
                Player::White,
            ),
        );

        match self.state {
            GameState::Turn(Player::White) => Ok(game_move),
            GameState::Turn(Player::Black) => Ok(self.reverse_move(&game_move)),
            _ => panic!("The game has ended already"),
        }
    }

    pub fn check_move<'a>(&self, game_move: Move) -> Result<AvailableMove, &'a str> {
        if let GameState::Won(_) = self.state {
            return Err("The game is already finished");
        };

        println!("Move: {:?}", game_move);

        // check if move is valid
        let available_moves = match self.get_allowed_moves_for(game_move.source) {
            Ok(available_moves) => available_moves,
            Err(msg) => return Err(msg),
        };

        println!("Available moves: {:?}", available_moves);

        if let Some(game_move) = Game::find_move_in_available(available_moves, game_move) {
            Ok(game_move)
        } else {
            Err("Illegal move")
        }
    }

    pub fn find_move_in_available(
        available_moves: Vec<AvailableMove>,
        game_move: Move,
    ) -> Option<AvailableMove> {
        // TODO: implement multi-captures
        let available_targets: Vec<&AvailableMove> = available_moves
            .iter()
            // take only non-multicapture moves
            .filter(|x| match x.move_type() {
                MoveType::MultiCapture(_) => false,
                MoveType::KingMultiCapture(_) => false,
                _ => true,
            })
            .collect();

        let available_move = available_targets.into_iter().find(|x| match x.move_type() {
            MoveType::Move(index) => index == &game_move.target,
            MoveType::Capture { target_index, .. } => target_index == &game_move.target,
            MoveType::KingMove(index) => index == &game_move.target,
            MoveType::KingCapture { target_index, .. } => target_index == &game_move.target,
            MoveType::KingMultiCapture(_) => false,
            MoveType::MultiCapture(_) => false,
        });

        available_move.cloned()
    }

    pub fn get_allowed_moves_for<'a>(&self, source: Index) -> Result<Vec<AvailableMove>, &'a str> {
        let pawn = match self.board.get_tile(source) {
            Ok(tile) => tile,
            Err(msg) => return Err(msg),
        };

        // Check if source tile is empty
        if pawn == Tile::Empty {
            return Err("Source is an empty tile");
        }

        let mut available_moves = Vec::new();

        match pawn {
            // Check for man moves
            Tile::White | Tile::Black => {
                println!("Checking diagonal moves");

                // Check diagonal moves
                if let Some(left_diagonal) = source.translate(-1, -1) {
                    if self.board.get_tile(left_diagonal) == Ok(Tile::Empty) {
                        available_moves
                            .push(AvailableMove::new(source, MoveType::Move(left_diagonal)));
                    }
                }

                if let Some(right_diagonal) = source.translate(1, -1) {
                    if self.board.get_tile(right_diagonal) == Ok(Tile::Empty) {
                        available_moves
                            .push(AvailableMove::new(source, MoveType::Move(right_diagonal)));
                    }
                }

                // Check for captures
                if let Ok(available_move) = self.check_capture_move(source, Direction::NE) {
                    available_moves.push(available_move);
                }
                if let Ok(available_move) = self.check_capture_move(source, Direction::NW) {
                    available_moves.push(available_move);
                }
                if let Ok(available_move) = self.check_capture_move(source, Direction::SE) {
                    available_moves.push(available_move);
                }
                if let Ok(available_move) = self.check_capture_move(source, Direction::SW) {
                    available_moves.push(available_move);
                }

                // Check for multi-captures
                // TODO: implement multi-captures, probably recursively
            }
            // Check for king moves
            Tile::WhiteKing | Tile::BlackKing => {
                // TODO: implement king move

                // TODO: implement king captures

                // TODO: implement king multi captures
            }
            Tile::Empty => panic!(INTERNAL_ERROR_MESSAGE),
        }

        Ok(available_moves)
    }

    fn check_capture_move(
        &self,
        source: Index,
        direction: Direction,
    ) -> Result<AvailableMove, &str> {
        // Check if source is a pawn on the board
        let source_tile = match self.board.get_tile(source) {
            Ok(source_tile) => source_tile,
            Err(msg) => return Err(msg),
        };

        // Check if source tile is not empty
        let player = match get_tile_owner(source_tile) {
            Some(player) => player,
            None => return Err("Source tile is empty"),
        };
        let enemy_player = get_enemy(player);

        // Get target tile index
        let target = match direction {
            Direction::NW => source.translate(-2, -2),
            Direction::NE => source.translate(2, -2),
            Direction::SE => source.translate(2, 2),
            Direction::SW => source.translate(-2, 2),
        };

        // Check if we could translate index
        let target_index = match target {
            None => return Err("Target tile is outside of the board"),
            Some(index) => index,
        };

        // Check if target is on the board
        let target_tile = match self.board.get_tile(target_index) {
            Ok(tile) => tile,
            Err(msg) => return Err(msg),
        };

        // Check if target tile is empty
        if target_tile != Tile::Empty {
            return Err("Target tile is not empty");
        }

        // Get captured tile index
        let captured_index = match direction {
            Direction::NW => source.translate(-1, -1),
            Direction::NE => source.translate(1, -1),
            Direction::SE => source.translate(1, 1),
            Direction::SW => source.translate(-1, 1),
        }
        // Captured tile is between target and source, so it has to be on the board
        .unwrap();

        // Captured tile is between target and source, so it has to be on the board
        let captured_tile = self.board.get_tile(captured_index).unwrap();

        // Check if captured tile belongs to enemy
        if get_tile_owner(captured_tile) != Some(enemy_player) {
            return Err("Tile to be captured does not belong to enemy");
        }

        Ok(AvailableMove::new(
            source,
            MoveType::Capture {
                target_index,
                captured_index,
            },
        ))
    }

    pub fn draw_board(&self) {
        let board = self.board.get_drawed_board();

        println!("{}", board);
    }

    fn reverse_move(&self, game_move: &Move) -> Move {
        let source = self.board.reverse_index(&game_move.source);
        let target = self.board.reverse_index(&game_move.target);

        Move::new(source, target)
    }

    pub fn draw_info(&self) {
        println!("Note that your terminal may change pawn and board apperance, here is reference:");

        self.board.draw_info();

        println!();
    }
}

#[derive(Debug, Clone, Copy)]
struct Move {
    // Source tile index
    source: Index,
    // Target tile index
    target: Index,
}

impl Move {
    pub fn new(source: Index, target: Index) -> Move {
        Move { source, target }
    }
}

// Change to isize to enable simpler Index math
#[derive(Debug, Copy, Clone, PartialEq)]
struct Index {
    // Player board orientation that the move is indexed from
    orientation: Player,
    // Horizontal index, 0-based
    x: usize,
    // Horizontal index, 0-based
    y: usize,
}

impl Index {
    /// Create new index, x is horizontal index, y is vertical. Indeces are 0-based
    pub fn new(x: usize, y: usize, orientation: Player) -> Index {
        Index { x, y, orientation }
    }

    /// Creates new index moved by (x, y)
    // change `Option` to `Result`
    pub fn translate(&self, x: isize, y: isize) -> Option<Index> {
        let x_new = usize::try_from((self.x as isize) + x);
        let y_new = usize::try_from((self.y as isize) + y);

        if x_new.is_err() || y_new.is_err() {
            return None;
        }

        let index_translated = Index::new(x_new.unwrap(), y_new.unwrap(), self.orientation);

        println!("Index translated: {:?}", index_translated);

        Some(index_translated)
    }
}

// TODO: move description doesn't support multi captures yet
#[derive(Debug, Clone)]
enum MoveType {
    Move(Index),
    Capture {
        target_index: Index,
        captured_index: Index,
    },
    MultiCapture(Vec<Index>),
    KingMove(Index),
    KingCapture {
        target_index: Index,
        captured_index: Index,
    },
    KingMultiCapture(Vec<Index>),
}

#[derive(Debug, Clone)]
struct AvailableMove {
    source: Index,
    move_type: MoveType,
}

impl AvailableMove {
    pub fn new(source: Index, move_type: MoveType) -> AvailableMove {
        AvailableMove { source, move_type }
    }

    pub fn source(&self) -> Index {
        self.source
    }

    pub fn move_type(&self) -> &MoveType {
        &self.move_type
    }
}
