// Terrain tests

extern crate chrono;
extern crate ncurses;
extern crate noise;
extern crate rand;

use chrono::*;
use ncurses::*;
use noise::{NoiseFn, Perlin};
use rand::Rng;

const KEY_QUIT: i32 = 'q' as i32;
const KEY_PAUSE: i32 = 'p' as i32;
const KEY_JUMP: i32 = 'w' as i32;

const PLAYER_CHAR: u32 = '$' as u32;
const OBSTACLE_CHAR: u32 = '#' as u32;

const JUMP_TO_MAX_HEIGHT_DIST: i32 = 3;
const IY: i32 = 6;
const IX: i32 = 1;
const PX: i32 = 23;

// Change these to alter terrain generation.
const MIN_FLAT: f64 = -0.46;
const MAX_FLAT: f64 = 0.46;
const X_STEP: f64 = 0.15;
const Y_STEP: f64 = 0.03;

const MAX_OBST_LENGHT: u32 = 5;
const MIN_OBST_DIST: u32 = 40;
const MIN_INCL_DIST: u32 = 2;

const MAX_SPEED: i64 = 40; // milliseconds update time
const SPEED_CHANGE_INTERVAL: u32 = 300;
const SPEED_MULT_CONST: f64 = 0.1;
const INITIAL_SPEED: i64 = 100;
const INITIAL_AIR_TIME: i32 = 7;

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
    remember_jump: bool,
}

impl Player {
    fn jump(&mut self, t: TerrainType) {
        if t == TerrainType::Up {
            self.remember_jump = true;
        } else if self.state == PlayerState::Running {
            self.state = PlayerState::Jumping;
        }
    }

    fn update_pos(
        &mut self,
        current_unit: &TerrainUnit,
        offset_y: i32,
        roffset_y: i32,
        max_air_time: i32,
    ) {
        match self.state {
            PlayerState::Jumping => {
                self.y_pos -= 1;
                self.air_dist += 1;

                if self.air_dist == JUMP_TO_MAX_HEIGHT_DIST {
                    self.state = PlayerState::MaxHeight;
                }

                if self.y_pos >= IY - roffset_y {
                    self.state = PlayerState::Running;
                }
            }
            PlayerState::MaxHeight => {
                if self.y_pos >= IY - roffset_y {
                    self.state = PlayerState::Running;
                    self.air_dist = 0;
                } else {
                    self.air_dist += 1;

                    if self.air_dist == max_air_time {
                        self.state = PlayerState::Falling;
                    }
                }
            }
            PlayerState::Falling => {
                if self.y_pos >= IY - roffset_y {
                    self.state = PlayerState::Running;
                    self.air_dist = 0;
                } else {
                    self.y_pos += 1;
                }
            }
            _ => {
                if self.remember_jump && current_unit.unit_type != TerrainType::Up{
                    self.state = PlayerState::Jumping;
                    self.remember_jump = false;
                } else {
                    self.y_pos = IY - roffset_y
                }
            },
        };

        if self.y_pos == current_unit.initial_y + offset_y && current_unit.obstacle {
            self.state = PlayerState::Dead;
        }
    }
}

fn generate_next_terrain_screen(
    last_unit: &TerrainUnit,
    last_incline_dist: &mut u32,
    last_obst_dist: &mut u32,
    screen_size: usize,
) -> Vec<TerrainUnit> {
    let mut t: Vec<TerrainUnit> = Vec::new();
    let perlin: Perlin = Perlin::new();

    let mut rng = rand::thread_rng();
    let n: f64 = rng.gen::<f64>();

    let mut last_type: TerrainType = last_unit.unit_type;
    let mut last_y: i32 = last_unit.initial_y;
    let mut last_obst: bool = last_unit.obstacle;

    let mut next_obst_len: u32 = rng.gen_range(2, MAX_OBST_LENGHT + 1);
    let mut obst_len: u32 = 0;

    for i in 0..screen_size {
        let v: f64 = perlin.get([X_STEP * i as f64 + n, Y_STEP * i as f64 + n]);

        if v <= MIN_FLAT && last_type != TerrainType::Up && !last_obst {
            t.push(TerrainUnit::new_down(last_y + 1));
            *last_obst_dist += 1;
            *last_incline_dist = 0;

            if obst_len != 0 {
                *last_obst_dist = 0;
            }
        } else if v >= MAX_FLAT && last_type != TerrainType::Down && !last_obst {
            t.push(TerrainUnit::new_up(last_y));
            *last_obst_dist += 1;
            *last_incline_dist = 0;

            if obst_len != 0 {
                *last_obst_dist = 0;
            }
        } else {
            let mut spawn_obst: bool = false;

            if *last_obst_dist > MIN_OBST_DIST
                && *last_incline_dist > MIN_INCL_DIST
                && obst_len < next_obst_len
            {
                spawn_obst = true;
                obst_len += 1;
            } else if obst_len == next_obst_len {
                obst_len = 0;
                *last_obst_dist = 0;
                next_obst_len = rng.gen_range(2, MAX_OBST_LENGHT + 1);
            }

            *last_obst_dist += 1;
            *last_incline_dist += 1;
            t.push(TerrainUnit::new_flat(last_y, spawn_obst));
        }

        if last_type == TerrainType::Up {
            t[i].initial_y -= 1;
        }

        last_type = t[i].unit_type;
        last_y = t[i].initial_y;
        last_obst = t[i].obstacle;
    }

    t
}

fn scroll_terrain(
    t: &mut Vec<TerrainUnit>,
    screen_dist: u32,
    last_incline_dist: &mut u32,
    last_obst_dist: &mut u32,
) -> u32 {
    t.remove(0);

    if screen_dist == COLS() as u32 / 3 {
        let last_unit = *t.last().unwrap();
        t.append(&mut generate_next_terrain_screen(
            &last_unit,
            last_incline_dist,
            last_obst_dist,
            COLS() as usize,
        ));

        return 1;
    }

    screen_dist + 1
}

fn draw(terrain: &Vec<TerrainUnit>, offset_y: i32, player: &Player, score: u32) {
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
    mvprintw(LINES() - 1, 0, &format!("Score: {}", score));
    refresh();
}

fn main() {
    initscr();
    raw();
    cbreak();
    nodelay(stdscr(), true);
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    loop {
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
            state: PlayerState::Idle,
            remember_jump: false,
        };

        let mut last_time = offset::Local::now();
        let mut screen_dist: u32 = 0;
        let mut last_incline_dist: u32 = 0;
        let mut last_obst_dist: u32 = 0;

        let mut offset_y: i32 = 0;
        let mut roffset_y: i32 = 0;

        let mut pause: bool = false;
        let mut playing: bool = true;
        let mut score: u32 = 0;

        let mut speed: i64 = INITIAL_SPEED;
        let mut speed_mult: f64 = 1.0;
        let mut max_air_time: i32 = INITIAL_AIR_TIME;

        draw(&terrain, offset_y, &player, score);
        mvprintw(LINES() / 2, COLS() / 2 - 12, "PRESS ANY KEY TO PLAY");

        while player.state == PlayerState::Idle {
            let key = getch();

            if key == KEY_QUIT {
                nocbreak();
                endwin();
                return;
            } else if key != -1 {
                player.state = PlayerState::Running;
            }
        }

        while playing {
            let key = getch();

            if key == KEY_QUIT {
                playing = false;
            } else if key == KEY_JUMP && !pause {
                player.jump(terrain[PX as usize].unit_type);
            } else if key == KEY_PAUSE && player.state != PlayerState::Dead {
                pause = !pause;
            }

            let t = offset::Local::now();
            if t >= last_time + Duration::milliseconds(speed) {
                if !pause && player.state != PlayerState::Dead {
                    screen_dist = scroll_terrain(
                        &mut terrain,
                        screen_dist,
                        &mut last_incline_dist,
                        &mut last_obst_dist,
                    );
                    last_time = t;

                    if player.state == PlayerState::Running && roffset_y != 0 {
                        let d = if roffset_y > 0 { 1 } else { -1 };
                        offset_y += d;
                        roffset_y -= d;
                    }

                    player.update_pos(&terrain[PX as usize], offset_y, roffset_y, max_air_time);
                    draw(&terrain, offset_y, &player, score);
                    score += 1;

                    roffset_y += match terrain[PX as usize].unit_type {
                        TerrainType::Flat => 0,
                        TerrainType::Down => -1,
                        TerrainType::Up => 1,
                    };

                    if score % SPEED_CHANGE_INTERVAL == 0 && speed > MAX_SPEED {
                        speed_mult -= SPEED_MULT_CONST;
                        speed = (INITIAL_SPEED as f64 * speed_mult) as i64; // linear
                                                                            // speed = (speed as f64 * speed_mult) as i64; // mon-linear
                        max_air_time =
                            INITIAL_AIR_TIME + (max_air_time as f64 * (1.0 - speed_mult)) as i32;
                    }
                } else if pause {
                    mvprintw(0, (COLS() / 2) - 3, "PAUSE");
                } else {
                    mvprintw(0, (COLS() / 2) - 3, "DEAD");
                    break;
                }
            }
        }

        mvprintw(
            2 * LINES() / 3,
            COLS() / 2 - 23,
            "PRESS 'JUMP' TO START AGAIN, 'QUIT' TO QUIT",
        );
        loop {
            let key = getch();
            if key == KEY_QUIT {
                nocbreak();
                endwin();
                return;
            } else if key == KEY_JUMP {
                break; // reset
            }
        }
    }
}
