extern crate chrono;
extern crate ncurses;
extern crate noise;
extern crate rand;

use chrono::*;
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

    let mut g = Game::new();

    loop {
        let mut terrain: t::Terrain = t::Terrain::new();
        let mut player: p::Player = p::Player::new();

        let mut last_time = offset::Local::now();

        // Start menu loop
        draw(&terrain, &player, &g);
        mvprintw(
            2 * LINES() / 3,
            COLS() / 2 - 23,
            "PRESS 'JUMP' TO START AGAIN, 'QUIT' TO QUIT",
        );

        while player.state == p::PlayerState::Idle {
            let key = getch();

            if key == KEY_QUIT {
                exit_config();
                return;
            } else if key != -1 {
                player.state = p::PlayerState::Running;
            }
        }

        // Main loop
        while g.playing {
            let key = getch();

            if key == KEY_QUIT {
                g.playing = false;
            } else if key == KEY_JUMP && !g.pause {
                player.jump(&terrain);
            } else if key == KEY_PAUSE && player.state != p::PlayerState::Dead {
                g.pause = !g.pause;
            }

            let t = offset::Local::now();
            if t >= last_time + Duration::milliseconds(g.speed) {
                if !g.pause && player.state != p::PlayerState::Dead {
                    last_time = t;

                    terrain.scroll_terrain();
                    terrain.offset(&player);

                    player.update_pos(&terrain, &g);
                    draw(&terrain, &player, &g);

                    terrain.roffset();
                    g.update_speed();
                    g.update_score();

                } else if g.pause {
                    mvprintw(0, (COLS() / 2) - 3, "PAUSE");
                } else {
                    mvprintw(0, (COLS() / 2) - 3, "DEAD");
                    break;
                }
            }
        }

        // Death / quit loop
        mvprintw(
            2 * LINES() / 3,
            COLS() / 2 - 23,
            "PRESS 'JUMP' TO START AGAIN, 'QUIT' TO QUIT",
        );

        loop {
            let key = getch();
            if key == KEY_QUIT {
                exit_config();
                return;
            } else if key == KEY_JUMP {
                break; // reset
            }
        }
    }
}
