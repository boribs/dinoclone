// Terrain tests

extern crate ncurses;
extern crate chrono;

use ncurses::*;
use chrono::*;

#[derive(Copy, Clone)]
struct TerrainTile {
    tile_char: u32,
}

#[derive(Copy, Clone)]
struct TerrainUnit {
    tiles: [TerrainTile; 3],
}


impl TerrainTile {
    fn new(c: char) -> TerrainTile {
        TerrainTile { tile_char: c as u32 }
    }
}

impl TerrainUnit {
    fn new() -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('.'),
                TerrainTile::new('.'),
            ]
        }
    }
    fn new2() -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('2'),
                TerrainTile::new('.'),
            ]
        }
    }
    fn new3() -> TerrainUnit {
        TerrainUnit {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('.'),
                TerrainTile::new('k'),
            ]
        }
    }
}


fn scroll_terrain(t: &mut Vec<TerrainUnit >) {
    let first = t.remove(0);
    t.push(first);
}

fn main() {
    initscr();
    raw();
    cbreak();
    nodelay(stdscr(), true);
    noecho();

    const IY: i32 = 4;
    const IX: i32 = 1;
    const PX: i32 = 20;

    let mut terrain: Vec<TerrainUnit> = Vec::new();

    for i in 0..COLS() - 1 {
        if i % 3 == 0 {
            terrain.push(TerrainUnit::new())
        } else if i % 2 == 0 {
            terrain.push(TerrainUnit::new2())
        } else {
            terrain.push(TerrainUnit::new3())
        }
    }

    let mut last_time = offset::Local::now();

    loop {
        for x in 0..terrain.len() {
            clear();
            mv(IY, IX);

            for i in 0..3 {
                for j in 0..=x {
                    addch(terrain[j].tiles[i].tile_char);
                }
                mv(IY + 1 + i as i32, IX);
            }
            mv(IY, IX);
        }
        mvprintw(IY - 1, PX, &"A");
        mvprintw(IY, PX, &"V");
        refresh();

        let c = getch();

        if c == 'q' as i32 {
            break
        }

        let t = offset::Local::now();
        if t >= last_time + Duration::milliseconds(100) {
            scroll_terrain(&mut terrain);
            last_time = t;
        }
    }

    nocbreak();
    endwin();
}
