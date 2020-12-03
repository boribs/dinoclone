// Terrain tests

extern crate chrono;
extern crate ncurses;
extern crate noise;
extern crate rand;

use chrono::*;
use ncurses::*;
use noise::{NoiseFn, Perlin};
use rand::Rng;

const PLAYER_CHAR: u32 = '$' as u32;
const OBSTACLE_CHAR: u32 = '#' as u32;

const MAX_JUMP_HEIGHT: i32 = 3;
const IY: i32 = 6;
const IX: i32 = 1;
const PX: i32 = 23;

// Change these to alter terrain generation.
const MIN_FLAT: f64 = -0.46;
const MAX_FLAT: f64 =  0.46;
const X_STEP: f64 = 0.15;
const Y_STEP: f64 = 0.03;

#[derive(Copy, Clone)]
struct TerrainTile {
    tile_char: u32,
}

#[derive(Copy, Clone, PartialEq)]
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
    air_dist: i32,
    state: PlayerState,
}

impl Player {
    fn jump(&mut self) {
        if self.state == PlayerState::Running {
            self.state = PlayerState::Jumping;
        }
    }

    fn update_pos(&mut self, roffset_y: i32) {
        match self.state {
            PlayerState::Jumping => {
                self.y_pos -= 1;

                if IY - self.y_pos == MAX_JUMP_HEIGHT {
                    self.state = PlayerState::MaxHeight;
                }

                if self.y_pos >= IY - roffset_y {
                    self.state = PlayerState::Running;
                    self.y_pos = IY;
                }
            }
            PlayerState::MaxHeight => {
                if self.y_pos >= IY - roffset_y {
                    self.state = PlayerState::Running;
                    self.air_dist = 0;
                    self.y_pos = IY;
                } else {
                    self.air_dist += 1;

                    if self.air_dist == 7 {
                        self.state = PlayerState::Falling;
                    }
                }
            }
            PlayerState::Falling => {
                if self.y_pos >= IY - roffset_y {
                    self.state = PlayerState::Running;
                    self.air_dist = 0;
                    self.y_pos = IY;
                } else {
                    self.y_pos += 1;
                }
            }
            _ => {}
        };
    }
}

fn generate_next_terrain_screen(last: &TerrainUnit, screen_size: usize) -> Vec<TerrainUnit> {
    let mut t: Vec<TerrainUnit> = Vec::new();
    let perlin: Perlin = Perlin::new();
    let n = rand::thread_rng().gen::<f64>();

    let mut last_type: TerrainType = last.unit_type;
    let mut last_y: i32 = last.initial_y;

    for i in 0..screen_size {
        let v: f64 = perlin.get([X_STEP * i as f64 + n, Y_STEP * i as f64 + n]);

        if v <= MIN_FLAT && last_type != TerrainType::Up {
            t.push(TerrainUnit::new_down(last_y + 1));
        } else if v >= MAX_FLAT && last_type != TerrainType::Down {
            t.push(TerrainUnit::new_up(last_y));
        } else {
            t.push(TerrainUnit::new_flat(last_y, false));
        }

        if last_type == TerrainType::Up {
            t[i].initial_y -= 1;
        }

        last_type = t[i].unit_type;
        last_y = t[i].initial_y;
    }

    t
}

fn scroll_terrain(t: &mut Vec<TerrainUnit>, screen_dist: u32) -> u32 {
    t.remove(0);

    if screen_dist == COLS() as u32 / 3 {
        let last = *t.last().unwrap();
        t.append(
            &mut generate_next_terrain_screen(
                &last,
                COLS() as usize
        ));

        return 1;
    }

    screen_dist + 1
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
        COLS() as usize / 3
    ]);
    let mut player: Player = Player {
        y_pos: IY,
        air_dist: 0,
        state: PlayerState::Running,
    };

    let mut last_time = offset::Local::now();
    let mut screen_dist: u32 = 0;

    let mut offset_y: i32 = 0;
    let mut roffset_y: i32 = 0;

    let mut pause: bool = false;

    while player.state != PlayerState::Dead {
        let c = getch();
        if c == 'q' as i32 {
            break;
        } else if c == 'w' as i32 {
            player.jump();
        } else if c == 'p' as i32 {
            pause = !pause;
        }

        let t = offset::Local::now();
        if t >= last_time + Duration::milliseconds(100) {
            if !pause {
                screen_dist = scroll_terrain(&mut terrain, screen_dist);
                last_time = t;

                roffset_y += match terrain[PX as usize].unit_type {
                    TerrainType::Flat => 0,
                    TerrainType::Up => 1,
                    TerrainType::Down => -1,
                };

                player.update_pos(roffset_y);

                if player.state == PlayerState::Running {
                    offset_y += roffset_y;
                    roffset_y = 0;
                }

                clear();

                mvprintw(0, 0, &format!("y pos:{}", player.y_pos));
                mvprintw(1, 0, &format!("y off:{}", offset_y));
                mvprintw(2, 0, &format!("y rof:{}", roffset_y));
                mvprintw(3, 0, &format!("state:{:?}", player.state));
                mvprintw(4, 0, &format!("tx:{}", terrain[PX as usize].initial_y));

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
            } else {
                mvprintw(0, (COLS() / 2) - 3, "PAUSE");
            }
        }
    }

    nocbreak();
    endwin();
}
