use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Icon, Input, Rect, Sheet, SheetAction, BLACK, BODY,
    CAPTION, PAPER,
};
use trailcarrier_core::{Blocked, Cell, Direction, Game, Status, LEVELS, SIZE};

const BOARD_X: i32 = 152;
const BOARD_Y: i32 = 335;
const CELL: i32 = 120;

pub struct View {
    pub game: Game,
    path: Option<PathBuf>,
    confirming_reset: bool,
    notice: Option<&'static str>,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let game = File::open(&path)
            .ok()
            .and_then(|file| {
                let mut bytes = Vec::new();
                file.take(4097).read_to_end(&mut bytes).ok()?;
                if bytes.len() > 4096 {
                    return None;
                }
                Game::resume(serde_json::from_slice(&bytes).ok()?)
            })
            .unwrap_or_default();
        Self {
            game,
            path: Some(path),
            confirming_reset: false,
            notice: None,
        }
    }

    pub fn preview() -> Self {
        Self {
            game: Game::new(),
            path: None,
            confirming_reset: false,
            notice: None,
        }
    }

    fn persist(&mut self) {
        if self.save().is_err() {
            self.notice = Some("Could not save this route.");
        }
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let width = primitive_app::WIDTH as i32;
        if self.confirming_reset {
            match reset_sheet(screen_rect(width, 0, 0, 1264, 1680)).hit(x, y) {
                Some(SheetAction::Confirm) => {
                    self.game.reset();
                    self.notice = None;
                    self.persist();
                }
                Some(SheetAction::Cancel) => {}
                None => return Change::None,
            }
            self.confirming_reset = false;
            return Change::Full;
        }
        if back_button(width).contains(x, y) {
            return Change::Exit;
        }
        if undo_button(width).contains(x, y) && self.game.undo() {
            self.notice = None;
            self.persist();
            return Change::Partial;
        }
        if action_button(width).contains(x, y) {
            if self.game.advance() {
                self.notice = None;
                self.persist();
                return Change::Full;
            }
            if self.game.can_undo() {
                self.confirming_reset = true;
                return Change::Full;
            }
        }
        if self.game.status() != Status::Playing {
            return Change::None;
        }
        let direction = Direction::ALL
            .into_iter()
            .enumerate()
            .find(|(index, _)| step_button(width, *index).contains(x, y))
            .map(|(_, direction)| direction)
            .or_else(|| {
                let board = board(width);
                if !board.contains(x, y) {
                    return None;
                }
                let column = ((x - board.x as f32) * SIZE as f32 / board.w as f32) as usize;
                let row = ((y - board.y as f32) * SIZE as f32 / board.h as f32) as usize;
                Direction::toward(self.game.head(), (column, row))
            });
        let Some(direction) = direction else {
            return Change::None;
        };
        self.notice = match self.game.play(direction) {
            Ok(()) => {
                self.notice = None;
                self.persist();
                return Change::Partial;
            }
            Err(Blocked::Wall) => Some("A wall blocks that step."),
            Err(Blocked::Trail) => Some("Your trail blocks that step."),
            Err(Blocked::ExitLocked) => Some("Collect the remaining parcels first."),
            Err(Blocked::Finished) => return Change::None,
        };
        Change::Partial
    }

    fn draw_board(&self, canvas: &mut Canvas, w: i32) {
        canvas.panel(board(w), 23, 3, PAPER);
        for row in 0..SIZE {
            for column in 0..SIZE {
                let x = BOARD_X + column as i32 * CELL;
                let y = BOARD_Y + row as i32 * CELL;
                let bounds = screen_rect(w, x + 2, y + 2, CELL - 4, CELL - 4);
                let cell = self
                    .game
                    .cell(column, row)
                    .expect("position is on the board");
                let can_step = !self.confirming_reset
                    && self.game.status() == Status::Playing
                    && Direction::toward(self.game.head(), (column, row)).is_some();
                let pressed = can_step && canvas.control_pressed(bounds, false);
                if pressed {
                    canvas.fill(bounds, BLACK);
                }
                let ink = if pressed { PAPER } else { BLACK };
                let paper = if pressed { BLACK } else { PAPER };
                match cell {
                    Cell::Wall => {
                        canvas.fill(bounds.inset(canvas.px(7)), ink);
                        for offset in [36, 55, 74] {
                            canvas.fill(screen_rect(w, x + 23, y + offset, 74, 5), paper);
                        }
                    }
                    Cell::Parcel => {
                        canvas.panel(bounds.inset(canvas.px(24)), 8, 4, paper);
                        canvas.fill(screen_rect(w, x + 55, y + 28, 10, 40), ink);
                        canvas.fill(screen_rect(w, x + 40, y + 55, 40, 7), ink);
                    }
                    Cell::Head => {
                        canvas.circle(bounds.inset(canvas.px(14)), ink);
                        canvas.circle(bounds.inset(canvas.px(39)), paper);
                        canvas.circle(bounds.inset(canvas.px(53)), ink);
                    }
                    Cell::Tail => canvas.circle(bounds.inset(canvas.px(31)), ink),
                    Cell::LockedExit | Cell::Exit => {
                        let open = cell == Cell::Exit;
                        let icon = bounds.inset(canvas.px(16));
                        canvas.panel(icon, 14, 4, if open { BLACK } else { PAPER });
                        let glyph = if open {
                            Icon {
                                stroke: &["M3 12H17 M12 7L17 12L12 17 M21 4V20"],
                                fill: &[],
                            }
                        } else {
                            Icon {
                                stroke: &["M6 6L18 18 M6 18L18 6"],
                                fill: &[],
                            }
                        };
                        let color = if open { PAPER } else { BLACK };
                        canvas.icon(centered_icon(icon, canvas.px(56)), glyph, color);
                    }
                    Cell::Floor => {}
                }
            }
        }
        // The grid is thin and stays in place when a step changes only two cells.
        let extent = CELL * SIZE as i32;
        for i in 1..SIZE {
            let offset = i as i32 * CELL;
            canvas.fill(
                screen_rect(w, BOARD_X + offset - 1, BOARD_Y + 2, 2, extent - 4),
                115,
            );
            canvas.fill(
                screen_rect(w, BOARD_X + 2, BOARD_Y + offset - 1, extent - 4, 2),
                115,
            );
        }
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let w = canvas.bounds().w;
        header(canvas, "Trail Carrier", "Done");
        let state = self.game.status();
        let moves = self.game.moves_left();
        let move_label = if moves == 1 { "move" } else { "moves" };
        canvas.card(screen_rect(w, 40, 163, 1184, 107));
        canvas.text(
            screen_rect(w, 69, 181, 318, 68),
            &format!("Route {} of {}", self.game.level() + 1, LEVELS.len()),
            CAPTION,
        );
        let parcels = self.game.parcels_left();
        let remaining = if parcels == 0 {
            "Exit open".to_string()
        } else if parcels == 1 {
            "1 parcel left".to_string()
        } else {
            format!("{parcels} parcels left")
        };
        canvas.text(
            screen_rect(w, 374, 180, 520, 70),
            &remaining,
            BODY.bold().center(),
        );
        canvas.text(
            screen_rect(w, 909, 181, 286, 68),
            &format!("{moves} {move_label} left"),
            BODY.bold().right(),
        );
        canvas.text(
            screen_rect(w, 40, 276, 1184, 52),
            "Move one square at a time. Collect parcels to open the exit.",
            CAPTION.center(),
        );
        self.draw_board(canvas, w);
        if state == Status::Playing {
            for (i, direction) in Direction::ALL.into_iter().enumerate() {
                let bounds = step_button(w, i);
                let pressed = canvas.control_pressed(bounds, false);
                canvas.panel(bounds, 16, 4, if pressed { BLACK } else { PAPER });
                canvas.icon(
                    centered_icon(bounds, canvas.px(54)),
                    arrow(direction),
                    if pressed { PAPER } else { BLACK },
                );
            }
        }
        let guidance = self.notice.unwrap_or(match state {
            Status::Complete => "Delivery complete.",
            Status::OutOfMoves => "Out of moves. Undo a step or reset.",
            Status::Playing if parcels == 0 => "Take the trail to the exit.",
            Status::Playing => "Tap a neighboring square or use the arrows.",
        });
        canvas.text(screen_rect(w, 40, 1431, 1184, 63), guidance, BODY.center());
        if self.game.can_undo() {
            canvas.button(undo_button(w), "Undo", false, false);
            let action = if state == Status::Complete {
                if self.game.level() + 1 == LEVELS.len() {
                    "Play again"
                } else {
                    "Next route"
                }
            } else {
                "Reset route"
            };
            canvas.button(action_button(w), action, state == Status::Complete, false);
        }
        if self.confirming_reset {
            reset_sheet(canvas.bounds()).draw(canvas, None);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::PageBack if self.confirming_reset => {
                self.confirming_reset = false;
                Change::Full
            }
            Input::PageBack => Change::Exit,
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.path {
            primitive_app::save(path, &serde_json::to_vec(self.game.progress())?)?;
        }
        Ok(())
    }
}

fn arrow(direction: Direction) -> Icon {
    let stroke: &[&str] = match direction {
        Direction::Left => &["M21 12H3 M10 5L3 12L10 19"],
        Direction::Right => &["M3 12H21 M14 5L21 12L14 19"],
        Direction::Up => &["M12 21V3 M5 10L12 3L19 10"],
        Direction::Down => &["M12 3V21 M5 14L12 21L19 14"],
    };
    Icon { stroke, fill: &[] }
}

fn centered_icon(bounds: Rect, size: i32) -> Rect {
    Rect::new(
        bounds.x + (bounds.w - size) / 2,
        bounds.y + (bounds.h - size) / 2,
        size,
        size,
    )
}

fn board(width: i32) -> Rect {
    let size = CELL * SIZE as i32;
    screen_rect(width, BOARD_X, BOARD_Y, size, size)
}

fn step_button(width: i32, index: usize) -> Rect {
    screen_rect(width, 152 + index as i32 * 243, 1308, 231, 109)
}

fn undo_button(width: i32) -> Rect {
    screen_rect(width, 40, 1531, 567, 112)
}

fn action_button(width: i32) -> Rect {
    screen_rect(width, 655, 1531, 569, 112)
}

fn reset_sheet(bounds: Rect) -> Sheet<'static> {
    Sheet::centered(
        bounds,
        "Reset this route?",
        "Your moves for this route will be cleared.",
        "Reset",
    )
}
