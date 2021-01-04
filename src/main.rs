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
    dinoclone::initialize_colors();

    loop {
        let mut terrain: t::Terrain = t::Terrain::new();
        let mut player: p::Player = p::Player {
            y_pos: IY,
            air_dist: 0,
            state: p::PlayerState::Idle,
            remember_jump: false,
        };

        let mut last_time = offset::Local::now();
        let mut screen_dist: u32 = 0;
        let mut last_incline_dist: u32 = 0;
        let mut last_obst_dist: u32 = 0;

        let mut speed: i64 = INITIAL_SPEED;
        let mut speed_mult: f64 = 1.0;
        let mut max_air_time: i32 = INITIAL_AIR_TIME;

        let mut offset_y: i32 = 0;
        let mut roffset_y: i32 = 0;

        let mut pause: bool = false;
        let mut playing: bool = true;

        let mut score: u32 = 0;

        draw(&terrain, offset_y, &player, score);
        mvprintw(LINES() / 2, COLS() / 2 - 12, "PRESS ANY KEY TO PLAY");

        while player.state == p::PlayerState::Idle {
            let key = getch();

            if key == KEY_QUIT {
                nocbreak();
                endwin();
                return;
            } else if key != -1 {
                player.state = p::PlayerState::Running;
            }
        }

        while playing {
            let key = getch();

            if key == KEY_QUIT {
                playing = false;
            } else if key == KEY_JUMP && !pause {
                player.jump(terrain.vec[PX as usize].unit_type);
            } else if key == KEY_PAUSE && player.state != p::PlayerState::Dead {
                pause = !pause;
            }

            let t = offset::Local::now();
            if t >= last_time + Duration::milliseconds(speed) {
                if !pause && player.state != p::PlayerState::Dead {
                    screen_dist = t::scroll_terrain(
                        &mut terrain.vec,
                        screen_dist,
                        COLS() as u32 / 3,
                        &mut last_incline_dist,
                        &mut last_obst_dist,
                    );
                    last_time = t;

                    if player.state == p::PlayerState::Running && roffset_y != 0 {
                        let d = if roffset_y > 0 { 1 } else { -1 };
                        offset_y += d;
                        roffset_y -= d;
                    }

                    player.update_pos(IY, &terrain.vec[PX as usize], offset_y, roffset_y, max_air_time);
                    draw(&terrain, offset_y, &player, score);
                    score += 1;

                    roffset_y += match terrain.vec[PX as usize].unit_type {
                        t::TerrainType::Flat => 0,
                        t::TerrainType::Down => -1,
                        t::TerrainType::Up => 1,
                    };

                    if score % SPEED_CHANGE_INTERVAL == 0 && speed > MAX_SPEED {
                        speed_mult -= SPEED_MULT_CONST;
                        speed = (INITIAL_SPEED as f64 * speed_mult) as i64; // linear
                                                                            // speed = (speed as f64 * speed_mult) as i64; // mon-linear
                        max_air_time =
                            INITIAL_AIR_TIME + (max_air_time as f64 * (1.0 - speed_mult)) as i32;
                    }
                } else if pause {
                    mvprintw(0, (COLS() / 2) - 3, "PAUSE");
                } else {
                    mvprintw(0, (COLS() / 2) - 3, "DEAD");
                    break;
                }
            }
        }

        mvprintw(
            2 * LINES() / 3,
            COLS() / 2 - 23,
            "PRESS 'JUMP' TO START AGAIN, 'QUIT' TO QUIT",
        );

        loop {
            let key = getch();
            if key == KEY_QUIT {
                nocbreak();
                endwin();
                return;
            } else if key == KEY_JUMP {
                break; // reset
            }
        }
    }
}
