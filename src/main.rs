// Terrain tests

extern crate chrono;
extern crate ncurses;
extern crate rand;

use chrono::*;
use ncurses::*;
use rand::seq::SliceRandom;

const PLAYER_CHAR: u32 = '$' as u32;
const OBSTACLE_CHAR: u32 = '#' as u32;
const MAX_JUMP_HEIGHT: i32 = 3;
const IY: i32 = 5;
const IX: i32 = 1;
const PX: i32 = 23;
const MIN_T_DIST: u32 = 10;
const MIN_O_DIST: u32 = 30;

#[derive(Copy, Clone)]
struct TerrainTile {
    tile_char: u32,
}

#[derive(Copy, Clone)]
enum TerrainType {
    Flat,
    Up,
    Down,
}

#[derive(Copy, Clone)]
struct TerrainUnit {
    tiles: [TerrainTile; 3],
    unit_type: TerrainType,
    initial_y: i32,
    obstacle: bool,
}

impl TerrainTile {
    fn new(c: char) -> TerrainTile {
        TerrainTile {
            tile_char: c as u32,
        }
    }
}

impl TerrainUnit {
    fn new_flat(iy: i32, obstacle: bool) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('.'),
                TerrainTile::new('.'),
            ],
            unit_type: TerrainType::Flat,
            initial_y: iy,
            obstacle: obstacle,
        }
    }

    fn new_up(iy: i32) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('/'),
                TerrainTile::new('.'),
                TerrainTile::new('.'),
            ],
            unit_type: TerrainType::Up,
            initial_y: iy,
            obstacle: false,
        }
    }

    fn new_down(iy: i32) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('\\'),
                TerrainTile::new('.'),
                TerrainTile::new('.'),
            ],
            unit_type: TerrainType::Down,
            initial_y: iy,
            obstacle: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum PlayerState {
    Idle,
    Running,
    Jumping,
    MaxHeight,
    Falling,
    Dead,
}

struct Player {
    y_pos: i32,
    state: PlayerState,
}

impl Player {
    fn jump(&mut self) {
        if self.state == PlayerState::Running {
            self.state = PlayerState::Jumping;
        }
    }

    fn update_pos(&mut self, air_dist: &mut i32) {
        match self.state {
            PlayerState::Jumping => {
                self.y_pos -= 1;

                if IY - self.y_pos == MAX_JUMP_HEIGHT {
                    self.state = PlayerState::MaxHeight;
                }
            }
            PlayerState::MaxHeight => {
                *air_dist += 1;

                if *air_dist == 7 {
                    self.state = PlayerState::Falling;
                }
            }
            PlayerState::Falling => {
                self.y_pos += 1;

                if self.y_pos == IY {
                    self.state = PlayerState::Running;
                    *air_dist = 0;
                }
            }
            _ => {}
        };
    }
}

fn generate_next_tile(
    previous: &TerrainUnit,
    dist_since_last_incl: u32,
    min_dist: u32,
    dist_since_last_obst: u32,
) -> TerrainUnit {
    let next_unit_type: TerrainType;
    let mut rng = rand::thread_rng();

    if dist_since_last_incl >= min_dist {
        next_unit_type = match previous.unit_type {
            TerrainType::Flat => *[TerrainType::Flat, TerrainType::Up, TerrainType::Down]
                .choose(&mut rng)
                .unwrap(),
            _ => TerrainType::Flat,
        }
    } else {
        next_unit_type = TerrainType::Flat;
    }

    let mut next_unit: TerrainUnit = match next_unit_type {
        TerrainType::Flat => {
            TerrainUnit::new_flat(previous.initial_y, *[true, false].choose(&mut rng).unwrap())
        }
        TerrainType::Up => TerrainUnit::new_up(previous.initial_y),
        TerrainType::Down => TerrainUnit::new_down(previous.initial_y + 1),
    };

    match previous.unit_type {
        TerrainType::Up => next_unit.initial_y -= 1,
        _ => {}
    };

    next_unit
}

fn scroll_terrain(
    t: &mut Vec<TerrainUnit>,
    dist_since_last_incl: u32,
    min_dist: u32,
    dist_since_last_obst: u32,
) -> u32 {
    let last: TerrainUnit = *t.last_mut().unwrap();
    let next: TerrainUnit =
        generate_next_tile(&last, dist_since_last_incl, min_dist, dist_since_last_obst);
    t.remove(0);
    t.push(next);

    match next.unit_type {
        TerrainType::Flat => dist_since_last_incl + 1,
        _ => 0,
    }
}

fn main() {
    initscr();
    raw();
    cbreak();
    nodelay(stdscr(), true);
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    let mut terrain: Vec<TerrainUnit> = vec![TerrainUnit::new_flat(IY, false); COLS() as usize];
    terrain.push(TerrainUnit {
        tiles: [
            TerrainTile::new('_'),
            TerrainTile::new('#'),
            TerrainTile::new('#'),
        ],
        unit_type: TerrainType::Flat,
        initial_y: IY,
        obstacle: false,
    });
    terrain.append(&mut vec![
        TerrainUnit::new_flat(IY, false);
        COLS() as usize / 6
    ]);
    let mut player: Player = Player {
        y_pos: IY,
        state: PlayerState::Running,
    };

    let mut air_dist: i32 = 0;
    let mut last_time = offset::Local::now();
    let mut dist_since_last_incl: u32 = 0;
    let mut offset_y: i32 = 0;
    let mut roffset_y: i32 = 0;

    while player.state != PlayerState::Dead {
        let c = getch();
        if c == 'q' as i32 {
            break;
        } else if c == 'w' as i32 {
            player.jump();
        }

        let t = offset::Local::now();
        if t >= last_time + Duration::milliseconds(100) {
            dist_since_last_incl =
                scroll_terrain(&mut terrain, dist_since_last_incl, MIN_T_DIST, 0);
            last_time = t;

            roffset_y += match terrain[PX as usize].unit_type {
                TerrainType::Flat => 0,
                TerrainType::Up => 1,
                TerrainType::Down => -1,
            };

            player.update_pos(&mut air_dist);

            if player.state == PlayerState::Running {
                offset_y += roffset_y;
                roffset_y = 0;
            }

            clear();
            mv(IY, IX);
            for j in 0..COLS() - 1 {
                for i in 0..3 {
                    mvaddch(
                        terrain[j as usize].initial_y + i + offset_y,
                        IX + j,
                        terrain[j as usize].tiles[i as usize].tile_char,
                    );
                }

                if terrain[j as usize].obstacle {
                    mvaddch(
                        terrain[j as usize].initial_y + offset_y,
                        IX + j,
                        OBSTACLE_CHAR,
                    );
                }
            }

            mvaddch(player.y_pos, PX, PLAYER_CHAR);
            refresh();
        }
    }

    nocbreak();
    endwin();
}
