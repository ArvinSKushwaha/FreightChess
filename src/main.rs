use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::io::Write;

use crate::ChessErr::{BadMove, IllegalCommand, InvalidIndexing};
use clap::{App, Arg};

const CHESS_PIECES: [char; 16] = [
    ' ', '♙', '♘', '♖', '♗', '♔', '♕', ' ', ' ', '♟', '♞', '♜', '♝', '♛', '♚', ' ',
];

// A bunch of constant that are really useful.
const EMPTY: u8 = 0;
const PAWN: u8 = 1;
const KNIGHT: u8 = 2;
const ROOK: u8 = 3;
const BISHOP: u8 = 4;
const QUEEN: u8 = 5;
const KING: u8 = 6;

const WHITE: u8 = 0;
const BLACK: u8 = 8;

const LEFT_MASK: u8 = 0xF0u8;
const RIGHT_MASK: u8 = 0x0Fu8;

// Define a bunch of useful functions to make the bit-manipulation sensible.
const GET_LEFT: fn(u8) -> u8 = |s: u8| (s & LEFT_MASK) >> 4;
const GET_RIGHT: fn(u8) -> u8 = |s: u8| s & RIGHT_MASK;

const GET_NUM: fn(u8) -> u8 = |s: u8| s & 0b0111;
const GET_COLOR: fn(u8) -> u8 = |s: u8| (s & BLACK) >> 3;

const SET_BLACK: fn(u8) -> u8 = |s: u8| (s | BLACK) * (s != EMPTY) as u8;
const SET_WHITE: fn(u8) -> u8 = |s: u8| s & !BLACK;

const GET_CELL_PAIR: fn(u8) -> (u8, u8) = |pair: u8| (GET_LEFT(pair), GET_RIGHT(pair));
const SET_CELL_PAIR: fn(u8, u8) -> u8 = |left: u8, right: u8| (left << 4) + right;
const SWAP_CELL: fn(u8) -> u8 = |pair: u8| SET_CELL_PAIR(GET_RIGHT(pair), GET_LEFT(pair));

const SET_LEFT: fn(u8, u8) -> u8 = |pair: u8, left: u8| SET_CELL_PAIR(left, GET_RIGHT(pair));
const SET_RIGHT: fn(u8, u8) -> u8 = |pair: u8, right: u8| SET_CELL_PAIR(GET_LEFT(pair), right);

// If the boolean is true, get the right piece, otherwise, get the left piece.
const GET_CELL_BOOLEAN: fn(u8, bool) -> u8 =
    |pair: u8, side: bool| GET_RIGHT(pair) * (side as u8) + GET_LEFT(pair) * (!side as u8);

// If the boolean is true, set the right piece, otherwise, set the left piece.
const SET_CELL_BOOLEAN: fn(u8, bool, u8) -> u8 = |pair: u8, side: bool, piece: u8| {
    SET_RIGHT(pair, piece) * (side as u8) + SET_LEFT(pair, piece) * (!side as u8)
};

pub fn get_app() -> App<'static> {
    App::new("Chess Engine")
        .version("0.1.0")
        .author("Arvin Kushwaha <arvin.singh.kushwaha@gmail.com>")
        .about(
            "A high-performance low-memory chess game 'platform' \
        for highly parallel and performant code. More options will \
        be added as the this program supports more chess options.",
        )
        .arg(
            Arg::new("play")
                .long("play")
                .short('p')
                .about("Starts new game."),
        )
}

#[derive(Debug)]
enum ChessErr {
    InvalidIndexing(&'static str),
    BadMove(&'static str),
    IllegalCommand(&'static str),
}

/// Represents the state of the board at any given point. Each byte is two cells.
struct ChessBoard {
    /// The chess board itself, 8x4 array of bytes (Each byte is a can store 2 pieces)
    /// Indexing the outer array returns the row. Each row contains 4 bytes representing pairs of two columns.
    /// In a standard depiction of the chess board, the white starting rows are located at the bottom.
    /// For ease of indexing (for me at least lol), those rows will be start at the 0th index.
    /// The columns will follow standard left-to-right convention.
    /// Each byte is composed of two sets of 4 bits:
    ///
    /// _ (Color of the piece) ___ (Type of piece)
    ///
    /// The piece values are as follows:
    /// - Empty: 0
    /// - Pawn: 1
    /// - Knight: 2
    /// - Rook: 3
    /// - Bishop: 4
    /// - Queen: 5
    /// - King: 6
    ///
    /// The color values are as follows (White Empty Squares and Black Empty Squares both have color 0):
    /// - White: 0
    /// - Black: 1
    board: [[u8; 4]; 8],
    moves: u16, // Theoretical maximum move count (with the FIDE limits) is somewhere around 6000, iirc?
}

impl ChessBoard {
    /// Creates new initialized ChessBoard.
    pub fn new() -> ChessBoard {
        ChessBoard {
            board: [
                [
                    SET_CELL_PAIR(SET_WHITE(ROOK), SET_WHITE(KNIGHT)),
                    SET_CELL_PAIR(SET_WHITE(BISHOP), SET_WHITE(QUEEN)),
                    SET_CELL_PAIR(SET_WHITE(KING), SET_WHITE(BISHOP)),
                    SET_CELL_PAIR(SET_WHITE(KNIGHT), SET_WHITE(ROOK)),
                ],
                [
                    SET_CELL_PAIR(SET_WHITE(PAWN), SET_WHITE(PAWN)),
                    SET_CELL_PAIR(SET_WHITE(PAWN), SET_WHITE(PAWN)),
                    SET_CELL_PAIR(SET_WHITE(PAWN), SET_WHITE(PAWN)),
                    SET_CELL_PAIR(SET_WHITE(PAWN), SET_WHITE(PAWN)),
                ],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [
                    SET_CELL_PAIR(SET_BLACK(PAWN), SET_BLACK(PAWN)),
                    SET_CELL_PAIR(SET_BLACK(PAWN), SET_BLACK(PAWN)),
                    SET_CELL_PAIR(SET_BLACK(PAWN), SET_BLACK(PAWN)),
                    SET_CELL_PAIR(SET_BLACK(PAWN), SET_BLACK(PAWN)),
                ],
                [
                    SET_CELL_PAIR(SET_BLACK(ROOK), SET_BLACK(KNIGHT)),
                    SET_CELL_PAIR(SET_BLACK(BISHOP), SET_BLACK(QUEEN)),
                    SET_CELL_PAIR(SET_BLACK(KING), SET_BLACK(BISHOP)),
                    SET_CELL_PAIR(SET_BLACK(KNIGHT), SET_BLACK(ROOK)),
                ],
            ],
            moves: 0,
        }
    }

    pub fn get_piece_at_bytes(&self, coord: &[u8]) -> Result<u8, ChessErr> {
        if !ChessBoard::is_valid_piece(coord) {
            return Err(InvalidIndexing("This is an invalid index"));
        }

        Ok(GET_CELL_BOOLEAN(
            self.board[((coord[1] & 0x0F) - 1) as usize][(((coord[0] - 1) & 0b0110) >> 1) as usize],
            ((coord[0] - 1) & 1) != 0,
        ))
    }

    // Remember, piece must be currently the rightmost piece (first four bits should be empty).
    pub fn set_piece_at_bytes(&mut self, coord: &[u8], piece: u8) -> Result<(), ChessErr> {
        if !ChessBoard::is_valid_piece(coord) {
            return Err(InvalidIndexing("This is an invalid index"));
        }

        self.board[((coord[1] & 0x0F) - 1) as usize][(((coord[0] - 1) & 0b0110) >> 1) as usize] =
            SET_CELL_BOOLEAN(
                self.board[((coord[1] & 0x0F) - 1) as usize]
                    [(((coord[0] - 1) & 0b0110) >> 1) as usize],
                ((coord[0] - 1) & 1) != 0,
                piece,
            );

        Ok(())
    }

    pub fn make_move(&mut self, move_from: &[u8], move_to: &[u8]) -> Result<(), ChessErr> {
        // Go ahead and perform the move for now.
        let piece = self.get_piece_at_bytes(move_from)?;
        self.set_piece_at_bytes(move_to, piece)?;
        todo!();
    }

    pub fn is_done(&self) -> bool {
        return false; // TODO: Actually give the board checkmate/draw testing.
    }

    fn is_valid_piece(coord: &[u8]) -> bool {
        !((coord.len() != 2)
            || (coord[0] & 0xF0 != 96)
            || !(1..=8).contains(&(coord[0] & 0x0F))
            || !(1..=8).contains(&(coord[1] & 0x0F))
            || (coord[1] & 0xF0 != 48))
    }
}

impl Display for ChessBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..8 {
            writeln!(
                f,
                "|{}|{}|{}|{}|{}|{}|{}|{}|",
                CHESS_PIECES[GET_LEFT(self.board[7 - i][0]) as usize],
                CHESS_PIECES[GET_RIGHT(self.board[7 - i][0]) as usize],
                CHESS_PIECES[GET_LEFT(self.board[7 - i][1]) as usize],
                CHESS_PIECES[GET_RIGHT(self.board[7 - i][1]) as usize],
                CHESS_PIECES[GET_LEFT(self.board[7 - i][2]) as usize],
                CHESS_PIECES[GET_RIGHT(self.board[7 - i][2]) as usize],
                CHESS_PIECES[GET_LEFT(self.board[7 - i][3]) as usize],
                CHESS_PIECES[GET_RIGHT(self.board[7 - i][3]) as usize],
            )?;
        }
        Ok(())
    }
}

fn print_game_tutorial() {
    let help = "Allowed commands:\n\
    - quit - Leaves game prompt\n\
    - exit - Leaves game prompt\n\
    - move [start]->[end] - expects [start] and [end] to be chessboard notation (in lowercase).";
    println!("{}", help);
}

/// Starts chess game prompt. (May be deprecated in a future version.)
fn play_chess() -> Result<(), ChessErr> {
    let mut board = ChessBoard::new();
    let stdin = std::io::stdin();
    let mut buff = String::new();
    while !board.is_done() {
        buff.clear();
        print!("{}\n>>> ", board);
        std::io::stdout().flush().unwrap();

        stdin
            .read_line(&mut buff)
            .expect("Yikes, something broke the prompt...");
        match buff.trim() {
            "help" => print_game_tutorial(),
            "exit" | "quit" => {
                return Ok(());
            }
            a => {
                let commands = a.split_whitespace().collect::<Vec<&str>>();
                if commands.len() != 2 {
                    return Err(IllegalCommand(
                        "Command does not exist or is not formatted properly.",
                    ));
                }
                match commands[0] {
                    "move" => {
                        if let Some((move_from, move_to)) = commands[1].split_once("->") {
                            board.make_move(move_from.as_bytes(), move_to.as_bytes())?;
                        } else {
                            return Err(IllegalCommand("Move formatting invalid."));
                        }
                    }
                    _ => {} // TODO: Add more functionality
                }
            }
        };
    }
    Ok(())
}

fn main() {
    // let mut app = Rc::new(RefCell::new(cmd_parse::get_app()));
    let app = RefCell::new(get_app());
    let matches = app.borrow_mut().get_matches_mut();
    let is_play = matches.is_present("play");

    if !is_play {
        app.borrow_mut()
            .print_help()
            .expect("Failed to print help.");

        return;
    }

    play_chess().unwrap_or_else(|x| {
        println!(
            "An error was encountered: {}",
            match x {
                InvalidIndexing(x) => x,
                BadMove(x) => x,
                IllegalCommand(x) => x,
            }
        )
    });
}

#[test]
fn test_get_piece_at() {
    assert!(ChessBoard::new()
        .get_piece_at_bytes("ab".as_bytes())
        .is_err());
    assert!(ChessBoard::new()
        .get_piece_at_bytes("12".as_bytes())
        .is_err());
    assert!(ChessBoard::new()
        .get_piece_at_bytes("i1".as_bytes())
        .is_err());
    assert!(ChessBoard::new()
        .get_piece_at_bytes("a9".as_bytes())
        .is_err());
    assert!(ChessBoard::new()
        .get_piece_at_bytes("a0".as_bytes())
        .is_err());

    let board = ChessBoard::new();
    let a1_piece = board
        .get_piece_at_bytes("a1".as_bytes())
        .expect("a1 failed");
    assert_eq!(GET_COLOR(a1_piece), 0);
    assert_eq!(GET_NUM(a1_piece), 3);
    assert_eq!(SET_WHITE(ROOK), a1_piece);
    let a2_piece = board
        .get_piece_at_bytes("a2".as_bytes())
        .expect("a2 failed");
    assert_eq!(SET_WHITE(PAWN), a2_piece);
    let a3_piece = board
        .get_piece_at_bytes("a3".as_bytes())
        .expect("a3 failed");
    assert_eq!(SET_WHITE(EMPTY), a3_piece);

    let a8_piece = board
        .get_piece_at_bytes("a8".as_bytes())
        .expect("a8 failed");
    assert_eq!(GET_COLOR(a8_piece), 1);
    assert_eq!(GET_NUM(a8_piece), 3);
    assert_eq!(SET_BLACK(ROOK), a8_piece);
    let a7_piece = board
        .get_piece_at_bytes("a7".as_bytes())
        .expect("a7 failed");
    assert_eq!(SET_BLACK(PAWN), a7_piece);
    let a6_piece = board
        .get_piece_at_bytes("a6".as_bytes())
        .expect("a6 failed");
    assert_eq!(SET_BLACK(EMPTY), a6_piece);

    for i in 1..=8 {
        for j in 1..=8 {
            assert!(ChessBoard::new()
                .get_piece_at_bytes(&[96 + i as u8, 48 + j as u8][..])
                .is_ok());
        }
    }
}
