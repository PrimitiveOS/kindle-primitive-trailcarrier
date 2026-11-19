//! Move one square per turn, grow the trail with each parcel, then reach the exit.

use serde::{Deserialize, Serialize};

pub const SIZE: usize = 8;
type Point = (usize, usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,
    Up,
    Down,
    Right,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Left, Self::Up, Self::Down, Self::Right];

    fn step(self, (x, y): Point) -> Option<Point> {
        match self {
            Self::Left => x.checked_sub(1).map(|x| (x, y)),
            Self::Up => y.checked_sub(1).map(|y| (x, y)),
            Self::Down => (y + 1 < SIZE).then_some((x, y + 1)),
            Self::Right => (x + 1 < SIZE).then_some((x + 1, y)),
        }
    }

    pub fn toward(from: Point, to: Point) -> Option<Self> {
        if from.0 >= SIZE || from.1 >= SIZE || to.0 >= SIZE || to.1 >= SIZE {
            return None;
        }
        Self::ALL
            .into_iter()
            .find(|&way| way.step(from) == Some(to))
    }
}

pub struct Level {
    pub limit: usize,
    map: [&'static str; SIZE],
}

pub const LEVELS: &[Level] = &[
    Level {
        limit: 10,
        map: [
            "........", "...##...", ".S..#...", ".##.#...", "....P...", "...##...", "......E.",
            "........",
        ],
    },
    Level {
        limit: 11,
        map: [
            ".....#..", ".S...P..", "#.......", "......P.", "..#...#E", "........", "..#.#.#.",
            "..#.#...",
        ],
    },
    Level {
        limit: 14,
        map: [
            "#.......", "........", "..E#P..#", ".##.....", "..#P....", ".#...#..", ".#...#..",
            ".S......",
        ],
    },
    Level {
        limit: 16,
        map: [
            "#..S....", "##...P..", "..#.#..#", "...E.#.P", ".#......", ".....#P.", "......##",
            "........",
        ],
    },
    Level {
        limit: 20,
        map: [
            "......#.", ".#......", ".#.P##..", ".##...P.", "S#....#.", ".#.P...E", ".....##.",
            "........",
        ],
    },
    Level {
        limit: 20,
        map: [
            ".#..#...", "..#.....", "...P..S#", ".#.P...#", "#..#..#.", "...#..#.", "#.E.....",
            "...#...P",
        ],
    },
    Level {
        limit: 23,
        map: [
            "#.......", ".P.E#..#", "#..#....", "......PP", "#...#.#.", ".....#.S", "...#.##.",
            ".#..P#..",
        ],
    },
    Level {
        limit: 24,
        map: [
            "#.......", "E......#", ".P#..##.", "#....#..", "P#..##..", "..S.....", ".#P...P#",
            "..###...",
        ],
    },
    Level {
        limit: 26,
        map: [
            "...E..#.", "##......", ".P.##...", "....S##.", "..#..#PP", "##.....#", "....P#..",
            ".#..#.#.",
        ],
    },
    Level {
        limit: 30,
        map: [
            "#...P#E.", "P....#..", "#..P.##.", "##..S...", "...#..#.", ".#.#....", "##...#P.",
            ".#...#..",
        ],
    },
];

impl Level {
    fn tile(&self, (x, y): Point) -> u8 {
        self.map[y].as_bytes()[x]
    }

    fn points(&self, kind: u8) -> impl Iterator<Item = Point> + '_ {
        self.map.iter().enumerate().flat_map(move |(y, row)| {
            row.bytes()
                .enumerate()
                .filter_map(move |(x, tile)| (tile == kind).then_some((x, y)))
        })
    }
}

struct Route {
    trail: Vec<Point>,
    parcels: Vec<Point>,
}

impl Route {
    fn new(level: &Level) -> Self {
        Self {
            trail: vec![level.points(b'S').next().expect("level has a start")],
            parcels: level.points(b'P').collect(),
        }
    }

    fn head(&self) -> Point {
        self.trail[self.trail.len() - 1]
    }

    fn step(&mut self, level: &Level, direction: Direction) -> Result<(), Blocked> {
        let next = direction.step(self.head()).ok_or(Blocked::Wall)?;
        match level.tile(next) {
            b'#' => return Err(Blocked::Wall),
            b'E' if !self.parcels.is_empty() => return Err(Blocked::ExitLocked),
            _ => {}
        }
        let parcel = self.parcels.iter().position(|&tile| tile == next);
        // The tail leaves its square on a regular step, while a parcel keeps the whole trail.
        let blocked_from = usize::from(parcel.is_none());
        if self.trail[blocked_from..].contains(&next) {
            return Err(Blocked::Trail);
        }
        if let Some(index) = parcel {
            self.parcels.swap_remove(index);
        } else {
            self.trail.remove(0);
        }
        self.trail.push(next);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Floor,
    Wall,
    Parcel,
    Tail,
    Head,
    LockedExit,
    Exit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blocked {
    Wall,
    Trail,
    ExitLocked,
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Playing,
    Complete,
    OutOfMoves,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Progress {
    level: usize,
    moves: Vec<Direction>,
}

pub struct Game {
    progress: Progress,
    route: Route,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            progress: Progress::default(),
            route: Route::new(&LEVELS[0]),
        }
    }

    pub fn resume(progress: Progress) -> Option<Self> {
        let level = LEVELS.get(progress.level)?;
        if progress.moves.len() > level.limit {
            return None;
        }
        let mut game = Self {
            progress: Progress {
                level: progress.level,
                moves: Vec::new(),
            },
            route: Route::new(level),
        };
        for direction in progress.moves {
            game.play(direction).ok()?;
        }
        Some(game)
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    pub fn level(&self) -> usize {
        self.progress.level
    }

    fn map(&self) -> &'static Level {
        &LEVELS[self.level()]
    }

    pub fn head(&self) -> Point {
        self.route.head()
    }

    pub fn parcels_left(&self) -> usize {
        self.route.parcels.len()
    }

    pub fn moves_left(&self) -> usize {
        self.map().limit - self.progress.moves.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.progress.moves.is_empty()
    }

    pub fn status(&self) -> Status {
        if self.map().tile(self.head()) == b'E' {
            Status::Complete
        } else if self.moves_left() == 0 {
            Status::OutOfMoves
        } else {
            Status::Playing
        }
    }

    pub fn cell(&self, x: usize, y: usize) -> Option<Cell> {
        if x >= SIZE || y >= SIZE {
            return None;
        }
        let point = (x, y);
        Some(match self.map().tile(point) {
            b'#' => Cell::Wall,
            b'E' if self.head() != point => {
                if self.parcels_left() == 0 {
                    Cell::Exit
                } else {
                    Cell::LockedExit
                }
            }
            _ if self.head() == point => Cell::Head,
            _ if self.route.trail.contains(&point) => Cell::Tail,
            _ if self.route.parcels.contains(&point) => Cell::Parcel,
            _ => Cell::Floor,
        })
    }

    pub fn play(&mut self, direction: Direction) -> Result<(), Blocked> {
        if self.status() != Status::Playing {
            return Err(Blocked::Finished);
        }
        self.route.step(self.map(), direction)?;
        self.progress.moves.push(direction);
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        if self.progress.moves.pop().is_none() {
            return false;
        }
        let level = self.map();
        self.route = Route::new(level);
        for &direction in &self.progress.moves {
            self.route
                .step(level, direction)
                .expect("accepted move replays");
        }
        true
    }

    pub fn reset(&mut self) {
        self.progress.moves.clear();
        self.route = Route::new(self.map());
    }

    pub fn advance(&mut self) -> bool {
        if self.status() != Status::Complete {
            return false;
        }
        self.progress.level = (self.level() + 1) % LEVELS.len();
        self.reset();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn direction(byte: u8) -> Direction {
        match byte {
            b'L' => Direction::Left,
            b'R' => Direction::Right,
            b'D' => Direction::Down,
            b'U' => Direction::Up,
            _ => panic!("invalid direction"),
        }
    }

    #[test]
    fn all_ten_routes_fit_their_budgets() {
        let solutions = [
            "RRDDRRDDR",
            "RRRRRDDRD",
            "RUURUURUULLD",
            "RRDRDDRDDLULLUL",
            "DDRRURUUUURRRDDRDD",
            "LLLDRRDDDRRDLULLLL",
            "UULLLLDDRDDLLUUUUULURR",
            "LLUDRRRRRRDLLLLUUULUUL",
            "LLULRDRDDDRURRURUUULLULL",
            "UULLLLRURRRDDLDRRDDRDRUUUUUUL",
        ];
        assert_eq!(solutions.len(), LEVELS.len());
        let mut game = Game::new();
        for (index, solution) in solutions.iter().enumerate() {
            assert_eq!(game.level(), index);
            assert_eq!(game.map().points(b'S').count(), 1);
            assert_eq!(game.map().points(b'E').count(), 1);
            assert!(game.map().map.iter().all(|row| row.len() == SIZE));
            for step in solution.bytes() {
                assert_eq!(game.play(direction(step)), Ok(()), "route {}", index + 1);
            }
            assert_eq!(game.status(), Status::Complete);
            assert!(game.advance());
        }
        assert_eq!(game.level(), 0);
    }

    #[test]
    fn walls_do_not_spend_turns_and_undo_puts_a_parcel_back() {
        let mut game = Game::new();
        assert_eq!(game.play(Direction::Down), Err(Blocked::Wall));
        assert_eq!(game.moves_left(), 10);
        for step in b"RRDDR" {
            game.play(direction(*step)).unwrap();
        }
        assert_eq!(game.parcels_left(), 0);
        assert_eq!(game.route.trail.len(), 2);
        let mut reopened = Game::resume(game.progress().clone()).unwrap();
        assert_eq!(reopened.head(), (4, 4));
        assert!(reopened.undo());
        assert_eq!(reopened.parcels_left(), 1);
        assert_eq!(reopened.route.trail.len(), 1);
        assert!(Game::resume(Progress {
            level: 0,
            moves: vec![Direction::Down]
        })
        .is_none());
    }

    #[test]
    fn entering_the_vacating_tail_is_legal_but_the_rest_blocks() {
        let level = &LEVELS[0];
        let mut route = Route {
            trail: vec![(0, 0), (1, 0), (1, 1), (0, 1)],
            parcels: vec![],
        };
        assert_eq!(route.step(level, Direction::Right), Err(Blocked::Trail));
        assert_eq!(route.step(level, Direction::Up), Ok(()));
        assert_eq!(route.head(), (0, 0));
    }
}
