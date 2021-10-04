use chess_engine::game_data::{Piece, PieceData, Position, BOARD_SIZE};
use ggez::graphics::{self, Color, Rect};
use ggez::{Context, GameError, GameResult};
use glam::*;

use crate::{Action, ActiveGame, MainState, RenderConfig, SpriteSheet};

pub(crate) const SCREEN_SIZE: (f32, f32) = (840f32, 840f32);

//const BOARD_SIZE: u8 = 8;
const BOARD_NUMBERS: [char; BOARD_SIZE as usize] = ['8', '7', '6', '5', '4', '3', '2', '1'];
const BOARD_LETTERS: [char; BOARD_SIZE as usize] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const BOARD_RENDER_SIZE: f32 = 720f32;
const BOARD_RENDER_TILE_SIZE: f32 = BOARD_RENDER_SIZE / BOARD_SIZE as f32;

const BOARD_RENDER_START: (f32, f32) = (
    SCREEN_SIZE.0 / 2.0 - BOARD_RENDER_SIZE / 2.0,
    SCREEN_SIZE.1 / 2.0 - BOARD_RENDER_SIZE / 2.0,
);

const BLACK_BOARD_COLOR: Color = Color {
    r: 0.4367,
    g: 0.31,
    b: 0.2533,
    a: 1.0,
};

const WHITE_BOARD_COLOR: Color = Color {
    r: 0.6591,
    g: 0.5125,
    b: 0.4284,
    a: 1.0,
};

pub const HIGHLIGHT_COLOR: Color = Color {
    r: 0.49,
    g: 0.88,
    b: 0.47,
    a: 0.5,
};

pub const MOVE_COLOR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 0.5,
};

const BACKGROUND_COLOR: Color = Color {
    r: 0.19,
    g: 0.18,
    b: 0.17,
    a: 1.0,
};

const BUTTON_COLOR: Color = Color {
    r: 0.29,
    g: 0.28,
    b: 0.27,
    a: 1.0,
};

const BUTTON_COLOR_SELECTED: Color = Color {
    r: 0.39,
    g: 0.38,
    b: 0.37,
    a: 1.0,
};

const CONFIRM_COLOR: Color = Color {
    r: 0.39,
    g: 0.88,
    b: 0.37,
    a: 1.0,
};

const ERROR_COLOR: Color = Color {
    r: 0.89,
    g: 0.38,
    b: 0.35,
    a: 1.0,
};

const BUTTON_RADIUS: f32 = 5.0;

pub fn get_square_from_screen(mouse: Vec2) -> Option<Position> {
    // because of margin this has to take place
    let zero_offset = mouse
        - (Vec2::new(
            SCREEN_SIZE.0 - BOARD_RENDER_SIZE,
            SCREEN_SIZE.1 - BOARD_RENDER_SIZE,
        ) / 2.0);
    if zero_offset.x < 0.0
        || zero_offset.x > BOARD_RENDER_SIZE
        || zero_offset.y < 0.0
        || zero_offset.y > BOARD_RENDER_SIZE
    {
        return None;
    }

    return Some(Position {
        x: (zero_offset.x / BOARD_RENDER_TILE_SIZE) as usize,
        y: (zero_offset.y / BOARD_RENDER_TILE_SIZE) as usize,
    });
}

fn get_piece_image(
    id: Piece,
    is_white: bool,
    square_is_black: bool,
    sprites: &SpriteSheet,
) -> &graphics::Image {
    if is_white {
        if id == Piece::Bishop && square_is_black {
            return &sprites.bishop_white_on_black_square;
        }
        return match id {
            Piece::Pawn => &sprites.pawn_white,
            Piece::Knight => &sprites.knight_white,
            Piece::Rook => &sprites.rook_white,
            Piece::King => &sprites.king_white,
            Piece::Queen => &sprites.queen_white,
            Piece::Bishop => &sprites.bishop_white,
            Piece::None => &sprites.bishop_white,
        };
    } else {
        if id == Piece::Bishop && square_is_black {
            return &sprites.bishop_black_on_black_square;
        }
        return match id {
            Piece::Pawn => &sprites.pawn_black,
            Piece::Knight => &sprites.knight_black,
            Piece::Rook => &sprites.rook_black,
            Piece::King => &sprites.king_black,
            Piece::Queen => &sprites.queen_black,
            Piece::Bishop => &sprites.bishop_black,
            Piece::None => &sprites.bishop_white,
        };
    }
}

fn get_render_pos(x: usize, y: usize) -> Vec2 {
    Vec2::new(
        BOARD_RENDER_START.0 + (x as f32) * BOARD_RENDER_TILE_SIZE,
        BOARD_RENDER_START.1 + (y as f32) * BOARD_RENDER_TILE_SIZE,
    )
}

pub(crate) fn render_clear(ctx: &mut Context) {
    graphics::clear(ctx, BACKGROUND_COLOR);
}

pub(crate) fn render_highlight(
    ctx: &mut Context,
    pos: Option<Position>,
    color: Color,
) -> GameResult<()> {
    let safe_pos = match pos {
        Some(p) => p,
        None => return Ok(()),
    };

    let square = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        Rect::new(0.0, 0.0, BOARD_RENDER_TILE_SIZE, BOARD_RENDER_TILE_SIZE),
        color,
    )?;
    graphics::draw(ctx, &square, (get_render_pos(safe_pos.x, safe_pos.y),))?;

    Ok(())
}

fn get_color(square_is_black: bool) -> Color {
    return if square_is_black {
        BLACK_BOARD_COLOR
    } else {
        WHITE_BOARD_COLOR
    };
}

fn render_round_rect(ctx: &mut Context, pos: Vec2, size: Vec2, color: Color) -> GameResult<()> {
    let square = graphics::Mesh::new_rounded_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        Rect::new(0.0, 0.0, size.x, size.y),
        BUTTON_RADIUS,
        color,
    )?;
    graphics::draw(ctx, &square, (pos,))?;

    Ok(())
}

/** Renders a rounded square with an image*/
pub fn render_button(
    ctx: &mut Context,
    pos: Vec2,
    size: Vec2,
    icon: &graphics::Image,
    color: Color,
) -> GameResult<()> {
    render_round_rect(ctx, pos, size, color)?;

    graphics::draw(
        ctx,
        icon,
        graphics::DrawParam::new()
            .dest(pos + size / 2.0)
            .offset(Vec2::new(0.5, 0.5)),
    )?;
    Ok(())
}

pub(crate) fn render_message(ctx: &mut Context, state: &MainState) -> Result<Action, GameError> {
    let msg = match &state.active_message {
        Some(m) => m,
        None => return Ok(Action::None),
    };

    let size = Vec2::new(300.0, 150.0);
    let pos = Vec2::new(SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0);
    let _err = render_round_rect(ctx, pos - size / 2.0, size, BUTTON_COLOR);
    let mut text = graphics::Text::new(msg.text.clone());
    let active_font = &state.render_config.fontsets[state.render_config.active_fontset_index];

    text.set_font(active_font.font, active_font.font_size);

    let _err2 = graphics::draw(
        ctx,
        &text,
        graphics::DrawParam::new()
            .dest(pos + Vec2::new(0.0, -50.0))
            .offset(Vec2::new(0.5, 0.0)),
    );

    let confirm_size = Vec2::new(60.0, 60.0);
    let confirm_pos = Vec2::new(pos.x - confirm_size.x * 1.5, pos.y);
    let cancel_pos = Vec2::new(pos.x + confirm_size.x / 2.0, pos.y);

    let mouse_x = state.input_staus.pos_x;
    let mouse_y = state.input_staus.pos_y;

    let is_hovering_confirm = is_inside_square(mouse_x, mouse_y, confirm_pos, confirm_size);
    let is_hovering_cancel = is_inside_square(mouse_x, mouse_y, cancel_pos, confirm_size);

    let mut confirm_color = CONFIRM_COLOR;
    let mut cancel_color = ERROR_COLOR;

    if is_hovering_confirm {
        confirm_color.a = 0.6;
    }
    if is_hovering_cancel {
        cancel_color.a = 0.6;
    }

    render_button(ctx, confirm_pos, confirm_size, &msg.confirm, confirm_color)?;
    render_button(ctx, cancel_pos, confirm_size, &msg.cancel, cancel_color)?;

    if is_hovering_confirm {
        return Ok(msg.confirm_value);
    } else if is_hovering_cancel {
        return Ok(msg.cancel_value);
    }

    Ok(Action::None)
}

fn is_inside_square(mouse_x: f32, mouse_y: f32, pos: Vec2, size: Vec2) -> bool {
    return mouse_x > pos.x
        && mouse_x < pos.x + size.x
        && mouse_y > pos.y
        && mouse_y < pos.y + size.y;
}

pub(crate) fn render_buttons(ctx: &mut Context, state: &MainState) -> Option<usize> {
    let icons: Vec<&graphics::Image> = vec![
        &state.render_config.icons.exit,
        // &state.render_config.icons.surrender,
        &state.render_config.icons.replay,
        &state.render_config.icons.settings,
    ];

    let start = SCREEN_SIZE.0 / 2.0 - 200.0;

    let mouse_x = state.input_staus.pos_x;
    let mouse_y = state.input_staus.pos_y;
    let mut hover_button = None;

    for x in 0..3 {
        let pos = Vec2::new(start + 150.0 * (x as f32), 5.0);
        let size = Vec2::new(100.0, 50.0);
        let is_hovering = is_inside_square(mouse_x, mouse_y, pos, size);
        if is_hovering {
            hover_button = Some(x);
        }
        let _err = render_button(
            ctx,
            pos,
            size,
            icons[x],
            if is_hovering {
                BUTTON_COLOR_SELECTED
            } else {
                BUTTON_COLOR
            },
        );
    }

    hover_button
}

/** Just renders the background board */
pub(crate) fn render_board(ctx: &mut Context) -> GameResult<()> {
    let bg_square = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        Rect::new(0.0, 0.0, BOARD_RENDER_SIZE, BOARD_RENDER_SIZE),
        WHITE_BOARD_COLOR,
    )?;
    graphics::draw(
        ctx,
        &bg_square,
        (Vec2::new(BOARD_RENDER_START.0, BOARD_RENDER_START.1),),
    )?;

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            if (x + y) % 2 == 0 {
                continue;
            }

            let square = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(0.0, 0.0, BOARD_RENDER_TILE_SIZE, BOARD_RENDER_TILE_SIZE),
                BLACK_BOARD_COLOR,
            )?;
            graphics::draw(ctx, &square, (get_render_pos(x, y),))?;
        }
    }

    Ok(())
}

pub(crate) fn render_numbers(ctx: &mut Context, config: &RenderConfig) -> GameResult<()> {
    let active_font = &config.fontsets[config.active_fontset_index];
    let x_add_offset = active_font.font_size.x / 4.0;
    let y_offset = Vec2::new(
        x_add_offset,
        BOARD_RENDER_TILE_SIZE - active_font.font_size.y,
    );
    let x_offset = Vec2::new(
        BOARD_RENDER_TILE_SIZE - active_font.font_size.x + x_add_offset,
        0.0,
    );
    for x in 0..BOARD_SIZE {
        let mut text = graphics::Text::new(BOARD_LETTERS[x as usize]);
        text.set_font(active_font.font, active_font.font_size);
        let dist = get_render_pos(x, BOARD_SIZE - 1);
        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(dist + y_offset)
                .color(get_color(x % 2 == 1)),
        )?;
    }

    for y in 0..BOARD_SIZE {
        let mut text = graphics::Text::new(BOARD_NUMBERS[y as usize]);
        text.set_font(active_font.font, active_font.font_size);
        let dist = get_render_pos(BOARD_SIZE - 1, y);
        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(dist + x_offset)
                .color(get_color(y % 2 == 1)),
        )?;
    }
    Ok(())
}

pub(crate) fn render_pieces(
    ctx: &mut Context,
    config: &RenderConfig,
    state: &mut ActiveGame,
) -> GameResult<()> {
    let mut selected_piece: Option<(Vec2, PieceData, bool)> = None;

    let active_sprites = &config.spritesets[config.active_sprites_index];

    let half_tile = Vec2::new(BOARD_RENDER_TILE_SIZE / 2.0, BOARD_RENDER_TILE_SIZE / 2.0);

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let piece_data = state.game.game.board[x][y];
            if piece_data.piece == Piece::None {
                continue;
            }
            let board_pos = Position { x, y };

            let is_on_white = (x + y) % 2 == 1;

            if state.hover_position.is_some()
                && state.selected_square.is_some()
                && board_pos == state.selected_square.unwrap()
            {
                let selected_render_dist = state.hover_position.unwrap() - half_tile;
                selected_piece = Some((selected_render_dist, piece_data, is_on_white));
                continue;
            }

            let dist = get_render_pos(x, y);

            graphics::draw(
                ctx,
                get_piece_image(
                    piece_data.piece,
                    piece_data.is_white,
                    is_on_white,
                    active_sprites,
                ),
                graphics::DrawParam::new()
                    .dest(dist + half_tile)
                    .offset(Vec2::new(0.5, 0.5)),
            )?;
        }
    }

    // because there does not exist a way to use z-index,
    // you will have to render in order for this to appear on top
    let (dist, piece, is_on_white) = match selected_piece {
        Some(p) => p,
        None => return Ok(()),
    };

    graphics::draw(
        ctx,
        get_piece_image(piece.piece, piece.is_white, is_on_white, active_sprites),
        graphics::DrawParam::new()
            .dest(dist + half_tile)
            .offset(Vec2::new(0.5, 0.5)),
    )?;

    Ok(())
}
