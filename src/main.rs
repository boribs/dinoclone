use dinoclone::*;
use ncurses::*;

fn main() {
    initscr();

    let h = get_highscore();
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    initialize_colors();

    Game::run(h);

    endwin();
}
