use ncurses::{attroff, attron, mv, mvaddch, COLS, COLOR_PAIR};
use noise::{NoiseFn, Perlin};
use rand::Rng;

use super::colors as c;

// Change these to alter terrain generation.
const MIN_FLAT: f64 = -0.46;
const MAX_FLAT: f64 = 0.46;
const X_STEP: f64 = 0.15;
const Y_STEP: f64 = 0.03;

const MAX_OBST_LENGHT: u32 = 5;
const MIN_OBST_DIST: u32 = 40;
const MIN_INCL_DIST: u32 = 2;

const OBSTACLE_CHAR: u32 = '#' as u32;

#[derive(Copy, Clone)]
pub struct TerrainTile {
    pub tile_char: u32,
    pub color_pair_id: i16,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TerrainType {
    Flat,
    Up,
    Down,
}

#[derive(Copy, Clone)]
pub struct TerrainUnit {
    pub tiles: [TerrainTile; 3],
    pub unit_type: TerrainType,
    pub initial_y: i32,
    pub obstacle: bool,
}

impl TerrainTile {
    pub fn new(c: char, color_id: i16) -> TerrainTile {
        TerrainTile {
            tile_char: c as u32,
            color_pair_id: color_id,
        }
    }
}

impl TerrainUnit {
    pub fn new_flat(iy: i32, obstacle: bool) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('_', c::PAIR_GREEN),
                TerrainTile::new('.', c::PAIR_WHITE),
                TerrainTile::new('.', c::PAIR_WHITE),
            ],
            unit_type: TerrainType::Flat,
            initial_y: iy,
            obstacle: obstacle,
        }
    }

    pub fn new_up(iy: i32) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('/', c::PAIR_GREEN),
                TerrainTile::new('.', c::PAIR_WHITE),
                TerrainTile::new('.', c::PAIR_WHITE),
            ],
            unit_type: TerrainType::Up,
            initial_y: iy,
            obstacle: false,
        }
    }

    pub fn new_down(iy: i32) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('\\', c::PAIR_GREEN),
                TerrainTile::new('.', c::PAIR_WHITE),
                TerrainTile::new('.', c::PAIR_WHITE),
            ],
            unit_type: TerrainType::Down,
            initial_y: iy,
            obstacle: false,
        }
    }
}

pub fn generate_next_terrain_screen(
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

pub fn scroll_terrain(
    t: &mut Vec<TerrainUnit>,
    screen_dist: u32,
    screen_update_dist: u32,
    last_incline_dist: &mut u32,
    last_obst_dist: &mut u32,
) -> u32 {
    t.remove(0);

    if screen_dist == screen_update_dist {
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

pub fn draw_terrain(t: &Vec<TerrainUnit>, offset_y: i32, iy: i32, ix: i32) {
    mv(iy, ix);
    for j in 0..COLS() - 1 {
        for i in 0..3 {
            attron(COLOR_PAIR(t[j as usize].tiles[i as usize].color_pair_id));
            mvaddch(
                t[j as usize].initial_y + i + offset_y,
                ix + j,
                t[j as usize].tiles[i as usize].tile_char,
            );
            attroff(COLOR_PAIR(t[j as usize].tiles[i as usize].color_pair_id));
        }

        if t[j as usize].obstacle {
            mvaddch(t[j as usize].initial_y + offset_y, ix + j, OBSTACLE_CHAR);
        }
    }
}
