extern crate chrono;
extern crate ncurses;
extern crate noise;
extern crate rand;
extern crate shellexpand;

use dinoclone::*;
use ncurses::*;

fn main() {
    initscr();

    let h = get_highscore();

    cbreak();
    nodelay(stdscr(), true);
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    initialize_colors();

    Game::run(h);

    nocbreak();
    endwin();
}
