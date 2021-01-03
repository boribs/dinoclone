use ncurses::*;

pub const PAIR_WHITE: i16 = 0;
pub const PAIR_GREEN: i16 = 1;
pub const PAIR_HIGHLIGHT: i16 = 2;


pub fn initialize_colors() {
    start_color();

    init_pair(PAIR_WHITE, COLOR_WHITE, COLOR_BLACK);
    init_pair(PAIR_GREEN, COLOR_GREEN, COLOR_BLACK);
    init_pair(PAIR_HIGHLIGHT, COLOR_YELLOW, COLOR_BLACK);
}
