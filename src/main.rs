// Terrain tests

extern crate ncurses;
extern crate chrono;
extern crate rand;

use ncurses::*;
use chrono::*;
use rand::seq::SliceRandom;

const IY: i32 = 4;
const IX: i32 = 1;

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
}


impl TerrainTile {
    fn new(c: char) -> TerrainTile {
        TerrainTile { tile_char: c as u32 }
    }
}

impl TerrainUnit {
    fn new_flat(iy: i32) -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('.'),
                TerrainTile::new('.'),
            ],
            unit_type: TerrainType::Flat,
            initial_y: iy,
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
        }
    }

    fn generate_next_tile(previous: &TerrainUnit, dist_since_last_incl: u32, min_dist: u32) -> TerrainUnit {
        let next_unit_type: TerrainType;
        let mut rng = rand::thread_rng();

        if dist_since_last_incl >= min_dist {
            next_unit_type = match previous.unit_type {
                TerrainType::Flat => {
                    *[
                        TerrainType::Flat,
                        TerrainType::Up,
                        TerrainType::Down,
                    ].choose(&mut rng).unwrap()
                },
                _ => TerrainType::Flat,
            }
        } else {
            next_unit_type = TerrainType::Flat;
        }

        let mut next_unit: TerrainUnit = match next_unit_type {
            TerrainType::Flat => TerrainUnit::new_flat(previous.initial_y),
            TerrainType::Up   => TerrainUnit::new_up(previous.initial_y),
            TerrainType::Down => TerrainUnit::new_down(previous.initial_y + 1),
        };

        match previous.unit_type {
            TerrainType::Up => next_unit.initial_y -= 1,
            _ => {},
        };

        next_unit
    }
}


fn scroll_terrain(t: &mut Vec<TerrainUnit>, dist_since_last_incl: u32, min_dist: u32) -> u32 {
    let last: TerrainUnit = *t.last_mut().unwrap();
    let next: TerrainUnit = TerrainUnit::generate_next_tile(
                                &last,
                                dist_since_last_incl,
                                min_dist
                            );
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

    let mut terrain: Vec<TerrainUnit> = vec!(TerrainUnit::new_flat(IY); (COLS() + COLS() / 3 ) as usize);
    let mut last_time = offset::Local::now();
    let mut dist_since_last_incl: u32 = 0;

    loop {
        let c = getch();
        if c == 'q' as i32 {
            break
        }

        let t = offset::Local::now();
        if t >= last_time + Duration::milliseconds(100) {
            dist_since_last_incl = scroll_terrain(
                                        &mut terrain,
                                        dist_since_last_incl,
                                        13
                                   );
            last_time = t;
            clear();
            mv(IY, IX);

            for j in 0..COLS() - 1 {
                for i in 0..3 {
                    mvaddch(
                        terrain[j as usize].initial_y + i,
                        IX + j,
                        terrain[j as usize].tiles[i as usize].tile_char
                    );
                }
            }

            refresh();
        }
    }

    nocbreak();
    endwin();
}
