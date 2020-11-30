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
struct Terrain {
    tiles: [TerrainTile; 3],
}


impl TerrainTile {
    fn new(c: char) -> TerrainTile {
        TerrainTile { tile_char: c as u32 }
    }
}

impl Terrain {
    fn new() -> Terrain {
        Terrain {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('.'),
                TerrainTile::new('.'),
            ]
        }
    }
    fn new2() -> Terrain {
        Terrain {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('2'),
                TerrainTile::new('.'),
            ]
        }
    }
    fn new3() -> Terrain {
        Terrain {
            tiles: [
                TerrainTile::new('_'),
                TerrainTile::new('.'),
                TerrainTile::new('k'),
            ]
        }
    }
}


fn scroll_terrain(t: &mut Vec<Terrain>) {
    let first = t.remove(0);
    t.push(first);
}

fn main() {
    initscr();
    raw();
    cbreak();
    nodelay(stdscr(), true);
    noecho();


    let mut terrain: Vec<Terrain> = Vec::new();
    for i in 0..COLS() - 1 {
        if i % 3 == 0 {
            terrain.push(Terrain::new())
        } else if i % 2 == 0 {
            terrain.push(Terrain::new2())
        } else {
            terrain.push(Terrain::new3())
        }
    }

    let mut last_time = offset::Local::now();

    loop {
        for x in 0..terrain.len() {
            clear();
            mv(3, 1);

            for i in 0..3 {
                for j in 0..=x {
                    addch(terrain[j].tiles[i].tile_char);
                }
                addch('\n' as u32);
                mv(4 + i as i32, 1);
            }
            addch('\n' as u32);
        }
        mvprintw(2, 20, &"A");
        mvprintw(3, 20, &"V");
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
