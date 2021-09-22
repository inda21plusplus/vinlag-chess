use std::collections::{HashMap, HashSet};

pub const BOARD_SIZE: usize = 8;
pub const BLACK_SPAWN: usize = 0;
pub const WHITE_SPAWN: usize = BOARD_SIZE - 1;
pub const WHITE_PAWN_Y: usize = BOARD_SIZE - 2;
pub const BLACK_PAWN_Y: usize = 1;
pub const EMPTY_PEICE: PieceData = PieceData {
    piece: Piece::None,
    is_white: false,
};

/*
pub const EMPTY_CASTLE: Castle = Castle {
    can_castle_king_side: false,
    can_castle_queen_side: false,
};*/

// logic
pub const BOARD_X_INPUT: [char; BOARD_SIZE] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub const BOARD_Y_INPUT: [char; BOARD_SIZE] = ['8', '7', '6', '5', '4', '3', '2', '1'];

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Piece {
    None,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PieceData {
    pub piece: Piece,
    pub is_white: bool,
}

pub struct ThreatMap {
    pub all_moves : HashSet<Position>,
    pub all_threats : HashSet<Position>,
    pub all_pinned : Vec<HashSet<Position>>,
    pub all_king_threats : HashSet<Position>,
    pub all_king_threats_full : HashSet<Position>,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Castle {
    pub can_castle_king_side: bool,
    pub can_castle_queen_side: bool,

    pub queen_side_rook : Position,
    pub king_side_rook : Position,
}

/** 0,0 is the top left; 8,8 is the bottom right */
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/**You might want to implement names for the players or want to extend with any metadata then add it here */
#[derive(Debug)]
pub struct Gameboard {
    /** all data used for the game logic */
    pub game : Game,
    /** 
    Used for 3 fold repetition Format in FEN string
    */
    pub same_board : HashMap<String, u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct Game {
    /** 0,0 is the top left; 8,8 is the bottom right */
    pub board: [[PieceData; BOARD_SIZE]; BOARD_SIZE],

    pub castle: [Castle; 2], // 2 players where 0 is the white player and 1 is the black player

    pub is_white_to_move: bool,

    /**
    This is recorded regardless of whether there is a pawn in position to make an en passant capture.
    */
    pub en_passant_position: Option<Position>,

    /**
    Halfmove clock: The number of halfmoves since the last capture or pawn advance,
    used for the fifty-move rule. https://en.wikipedia.org/wiki/Fifty-move_rule
    */
    pub half_move_clock: u16,
    /**
    Fullmove number: The number of the full move. It starts at 1, and is incremented after Black's move.
    */
    pub full_move_clock: u16,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vector2 {
    pub x: i8,
    pub y: i8,
}

pub(crate) struct Moveset {
    pub(crate) regular_moves: &'static [Vector2],
    pub(crate) inf_moves: &'static [Vector2],
}

pub enum WinStatus {
    WhiteWon,
    BlackWon,
    Tie,
    Nothing,
}

const DIAGONAL_MOVESET: &'static [Vector2; 4] = &[
    Vector2 { x: 1, y: 1 },
    Vector2 { x: -1, y: -1 },
    Vector2 { x: -1, y: 1 },
    Vector2 { x: 1, y: -1 },
];

const HORIZONTAL_MOVESET: &'static [Vector2; 4] = &[
    Vector2 { x: 0, y: 1 },
    Vector2 { x: 0, y: -1 },
    Vector2 { x: -1, y: 0 },
    Vector2 { x: 1, y: 0 },
];

const BOTH_MOVESET: &'static [Vector2; 8] = &[
    Vector2 { x: 0, y: 1 },
    Vector2 { x: 0, y: -1 },
    Vector2 { x: -1, y: 0 },
    Vector2 { x: 1, y: 0 },
    Vector2 { x: 1, y: 1 },
    Vector2 { x: -1, y: -1 },
    Vector2 { x: -1, y: 1 },
    Vector2 { x: 1, y: -1 },
];

const KNIGHT_MOVESET: &'static [Vector2; 8] = &[
    Vector2 { x: 2, y: 1 },
    Vector2 { x: 1, y: 2 },
    Vector2 { x: -2, y: 1 },
    Vector2 { x: -1, y: 2 },
    Vector2 { x: 2, y: -1 },
    Vector2 { x: 1, y: -2 },
    Vector2 { x: -2, y: -1 },
    Vector2 { x: -1, y: -2 },
];

const EMPTY_MOVESET: &'static [Vector2; 0] = &[];

/** GET MOVESET, WONT WORK FOR PAWN */
pub(crate) fn get_moveset(piece: Piece) -> Moveset {
    match piece {
        Piece::None => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
        Piece::Pawn => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
        Piece::Knight => Moveset {
            regular_moves: KNIGHT_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
        Piece::Bishop => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: DIAGONAL_MOVESET,
        },
        Piece::Rook => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: HORIZONTAL_MOVESET,
        },
        Piece::Queen => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: BOTH_MOVESET,
        },
        Piece::King => Moveset {
            regular_moves: BOTH_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
    }
}
