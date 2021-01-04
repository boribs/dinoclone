extern crate chrono;
extern crate ncurses;
extern crate noise;
extern crate rand;

use ncurses::*;

use dinoclone::*;

fn main() {
    initscr();
    raw();
    cbreak();
    nodelay(stdscr(), true);
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    initialize_colors();

    Game::run();

    nocbreak();
    endwin();
}
