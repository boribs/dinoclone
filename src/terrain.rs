use ncurses::{attroff, attron, mv, mvaddch, COLOR_PAIR, COLS};
use noise::{NoiseFn, Perlin};
use rand::Rng;

use crate::*;

// Change these to alter terrain generation.
const MIN_FLAT: f64 = -0.46;
const MAX_FLAT: f64 = 0.46;
const X_STEP: f64 = 0.15;
const Y_STEP: f64 = 0.03;

const MIN_OBST_LENGTH: u32 = 3;
const MAX_OBST_LENGTH: u32 = 6;

const MIN_OBST_DIST: u32 = 7;
const MAX_OBST_DIST: u32 = 70;
const MIN_INCL_DIST: u32 = 2;

const OBSTACLE_CHAR: u32 = '#' as u32;

#[derive(Copy, Clone)]
pub struct TerrainTile {
    pub tile_char: u32,
    color_pair_id: i16,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TerrainType {
    Flat = 0,
    Up = 1,
    Down = -1,
}

#[derive(Copy, Clone)]
pub struct TerrainUnit {
    pub tiles: [TerrainTile; 3],
    pub unit_type: TerrainType,
    pub initial_y: i32,
    pub obstacle: bool,
}

pub struct Terrain {
    pub vec: Vec<TerrainUnit>,
    pub offset_y: i32,
    pub roffset_y: i32,
    last_incl_dist: u32,
    last_obst_dist: u32,
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
                TerrainTile::new('_', PAIR_GREEN),
                TerrainTile::new('.', PAIR_WHITE),
                TerrainTile::new('.', PAIR_WHITE),
            ],
            unit_type: TerrainType::Flat,
            initial_y: iy,
            obstacle: obstacle,
        }
    }

    pub fn new_up(iy: i32) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('/', PAIR_GREEN),
                TerrainTile::new('.', PAIR_WHITE),
                TerrainTile::new('.', PAIR_WHITE),
            ],
            unit_type: TerrainType::Up,
            initial_y: iy,
            obstacle: false,
        }
    }

    pub fn new_down(iy: i32) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('\\', PAIR_GREEN),
                TerrainTile::new('.', PAIR_WHITE),
                TerrainTile::new('.', PAIR_WHITE),
            ],
            unit_type: TerrainType::Down,
            initial_y: iy,
            obstacle: false,
        }
    }
}

impl Terrain {
    pub fn new() -> Self {
        let mut terrain_vec: Vec<TerrainUnit> =
            vec![TerrainUnit::new_flat(IY, false); COLS() as usize];

        terrain_vec.push(t::TerrainUnit {
            tiles: [
                t::TerrainTile::new('_', PAIR_GREEN),
                t::TerrainTile::new('#', PAIR_WHITE),
                t::TerrainTile::new('#', PAIR_WHITE),
            ],
            unit_type: TerrainType::Flat,
            initial_y: IY,
            obstacle: false,
        });

        terrain_vec.append(&mut vec![
            TerrainUnit::new_flat(IY, false);
            COLS() as usize / 3
        ]);

        Terrain {
            vec: terrain_vec,
            last_incl_dist: 0,
            last_obst_dist: 0,
            offset_y: 0,
            roffset_y: 0,
        }
    }

    pub fn draw_terrain(&self) {
        mv(IY, IX);
        for j in 0..COLS() - 1 {
            for i in 0..3 {
                attron(COLOR_PAIR(
                    self.vec[j as usize].tiles[i as usize].color_pair_id,
                ));
                mvaddch(
                    self.vec[j as usize].initial_y + i + self.offset_y,
                    IX + j,
                    self.vec[j as usize].tiles[i as usize].tile_char,
                );
                attroff(COLOR_PAIR(
                    self.vec[j as usize].tiles[i as usize].color_pair_id,
                ));
            }

            if self.vec[j as usize].obstacle {
                attron(COLOR_PAIR(PAIR_RED));
                mvaddch(
                    self.vec[j as usize].initial_y + self.offset_y,
                    IX + j,
                    OBSTACLE_CHAR,
                );
                attroff(COLOR_PAIR(PAIR_RED));
            }
        }
    }

    pub fn generate_next_terrain_screen(&mut self, g: &mut Game) {
        let mut t: Vec<TerrainUnit> = Vec::new();
        let perlin: Perlin = Perlin::new();

        let mut rng = rand::thread_rng();
        let n: f64 = rng.gen::<f64>();

        let last_unit: &TerrainUnit = self.vec.last().unwrap();
        let mut last_type: TerrainType = last_unit.unit_type;
        let mut last_y: i32 = last_unit.initial_y;
        let mut last_obst: bool = last_unit.obstacle;

        let mut next_obst_len: u32 = rng.gen_range(MIN_OBST_LENGTH, MAX_OBST_LENGTH);
        let mut next_obst_dist: u32 = rng.gen_range(MIN_OBST_DIST, MAX_OBST_DIST);
        let mut obst_len: u32 = 0;

        for i in 0..g.screen_update_dist as usize + 1 {
            let v: f64 = perlin.get([X_STEP * i as f64 + n, Y_STEP * i as f64 + n]);

            if v <= MIN_FLAT && last_type != TerrainType::Up && !last_obst {
                t.push(TerrainUnit::new_down(last_y + 1));
                self.last_obst_dist += 1;
                self.last_incl_dist = 0;

                if obst_len != 0 {
                    self.last_obst_dist = 0;
                }
            } else if v >= MAX_FLAT && last_type != TerrainType::Down && !last_obst {
                t.push(TerrainUnit::new_up(last_y));
                self.last_obst_dist += 1;
                self.last_incl_dist = 0;

                if obst_len != 0 {
                    self.last_obst_dist = 0;
                }
            } else {
                let mut spawn_obst: bool = false;

                if self.last_obst_dist > next_obst_dist
                    && self.last_incl_dist > MIN_INCL_DIST
                    && obst_len < next_obst_len
                {
                    spawn_obst = true;
                    obst_len += 1;
                } else if obst_len == next_obst_len {
                    obst_len = 0;
                    self.last_obst_dist = 0;
                    next_obst_len = rng.gen_range(MIN_OBST_LENGTH, MAX_OBST_LENGTH);
                    next_obst_dist = rng.gen_range(MIN_OBST_DIST, MAX_OBST_DIST);
                }

                self.last_obst_dist += 1;
                self.last_incl_dist += 1;
                t.push(TerrainUnit::new_flat(last_y, spawn_obst));
            }

            if last_type == TerrainType::Up {
                t[i].initial_y -= 1;
            }

            last_type = t[i].unit_type;
            last_y = t[i].initial_y;
            last_obst = t[i].obstacle;
        }

        g.screen_count += 1;

        if g.highscore / (g.screen_update_dist + 1) as u32 == g.screen_count {
            let j: usize = (g.highscore % (g.screen_update_dist + 1)) as usize;

            t[j].tiles[0].color_pair_id = PAIR_WHITE;
            t[j].tiles[1].tile_char = '!' as u32;
            t[j].tiles[2].tile_char = '!' as u32;
        }


        if last_obst {
            self.last_obst_dist = 0;
        }
        self.vec.append(&mut t);
    }

    pub fn scroll_terrain(&mut self, g: &mut Game) {
        self.vec.remove(0);

        if g.screen_dist == g.screen_update_dist {
            self.generate_next_terrain_screen(g);
            g.screen_dist = 0;
            return;
        }

        g.screen_dist += 1;
    }

    pub fn offset(&mut self, p: &p::Player) {
        if p.state == p::PlayerState::Running && self.roffset_y != 0 {
            let d = if self.roffset_y > 0 { 1 } else { -1 };
            self.offset_y += d;
            self.roffset_y -= d;
        }
    }

    pub fn roffset(&mut self) {
        self.roffset_y += self.vec[PX as usize].unit_type as i32;
    }
}
