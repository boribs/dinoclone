use ncurses::*;
pub mod player;
pub mod terrain;

pub use player as p;
pub use terrain as t;

// Color stuff
pub const PAIR_WHITE: i16 = 0;
pub const PAIR_GREEN: i16 = 1;
pub const PAIR_YELLOW: i16 = 2;
pub const PAIR_RED: i16 = 3;
pub const PAIR_BLUE: i16 = 4;

pub const KEY_QUIT: i32 = 'q' as i32;
pub const KEY_PAUSE: i32 = 'p' as i32;
pub const KEY_JUMP: i32 = 'j' as i32;

pub const IY: i32 = 6;
pub const IX: i32 = 1;
pub const PX: i32 = 23;

pub const MAX_SPEED: i64 = 40; // milliseconds update time
pub const SPEED_CHANGE_INTERVAL: u32 = 300;
pub const SPEED_MULT_CONST: f64 = 0.1;
pub const INITIAL_SPEED: i64 = 100;
pub const INITIAL_AIR_TIME: i32 = 7;

pub fn initialize_colors() {
    start_color();

    init_pair(PAIR_WHITE, COLOR_WHITE, COLOR_BLACK);
    init_pair(PAIR_GREEN, COLOR_GREEN, COLOR_BLACK);
    init_pair(PAIR_YELLOW, COLOR_YELLOW, COLOR_BLACK);
    init_pair(PAIR_RED, COLOR_RED, COLOR_BLACK);
    init_pair(PAIR_BLUE, COLOR_BLUE, COLOR_BLACK);
}

pub fn draw(terrain: &t::Terrain, player: &p::Player, score: u32) {
    clear();
    terrain.draw_terrain();
    p::draw_player(player, PX);

    mvprintw(LINES() - 1, 0, &format!("Score: {}", score));
    refresh();
}
