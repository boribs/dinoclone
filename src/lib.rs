use chrono::*;
use ncurses::*;
use std::fs;
use std::io::ErrorKind;
use std::io::Write;

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

pub const IY: i32 = 9;
pub const IX: i32 = 1;
pub const PX: i32 = 23;

pub const MAX_SPEED: i64 = 40; // milliseconds update time
pub const SPEED_CHANGE_INTERVAL: u32 = 300;
pub const SPEED_MULT_CONST: f64 = 0.1;
pub const INITIAL_SPEED: i64 = 100;
pub const INITIAL_AIR_TIME: i32 = 7;

const SAVE_FILE_PATH: &str = "~/.dinoclone";

pub fn initialize_colors() {
    start_color();

    init_pair(PAIR_WHITE, COLOR_WHITE, COLOR_BLACK);
    init_pair(PAIR_GREEN, COLOR_GREEN, COLOR_BLACK);
    init_pair(PAIR_YELLOW, COLOR_YELLOW, COLOR_BLACK);
    init_pair(PAIR_RED, COLOR_RED, COLOR_BLACK);
    init_pair(PAIR_BLUE, COLOR_BLUE, COLOR_BLACK);
}

pub fn draw(terrain: &t::Terrain, player: &p::Player, game_data: &Game) {
    clear();
    terrain.draw_terrain();
    player.draw_player();

    mvprintw(LINES() - 2, 0, &format!("Score: {}", game_data.score));
    mvprintw(
        LINES() - 1,
        0,
        &format!("Highscore: {}", game_data.highscore),
    );
    refresh();
}

pub struct Game {
    pub playing: bool,
    pub pause: bool,
    pub score: u32,
    pub speed: i64,
    pub max_air_time: i32,
    pub highscore: u32,
    speed_mult: f64,
    tile_count: u32,
}

impl Game {
    pub fn new(highscore: u32) -> Self {
        Game {
            playing: true,
            pause: false,
            score: 0,
            speed: INITIAL_SPEED,
            max_air_time: INITIAL_AIR_TIME,
            highscore: highscore,
            speed_mult: 1.0,
            tile_count: 0,
        }
    }

    pub fn update_speed(&mut self) {
        if self.score != 0 && self.score % SPEED_CHANGE_INTERVAL == 0 && self.speed > MAX_SPEED {
            self.speed_mult -= SPEED_MULT_CONST;
            self.speed = (INITIAL_SPEED as f64 * self.speed_mult) as i64; // linear
                                                                          // speed = (speed as f64 * speed_mult) as i64; // non-linear
            self.max_air_time =
                INITIAL_AIR_TIME + (self.max_air_time as f64 * (1.0 - self.speed_mult)) as i32;
        }
    }

    pub fn update_score(&mut self) {
        if self.tile_count < (COLS() - PX) as u32 {
            self.tile_count += 1;
        } else {
            self.score += 1;
        }
    }

    pub fn run(highscore: u32) {
        nodelay(stdscr(), true);

        loop {
            let mut g = Game::new(highscore);
            let mut terrain: t::Terrain = t::Terrain::new();
            let mut player: p::Player = p::Player::new();

            let mut last_time = offset::Local::now();

            // Start menu loop
            draw(&terrain, &player, &g);
            mvprintw(
                2 * LINES() / 3,
                COLS() / 2 - 23,
                "PRESS 'JUMP' TO START, 'QUIT' TO QUIT",
            );

            while player.state == p::PlayerState::Idle {
                let key = getch();

                if key == KEY_QUIT {
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

                        terrain.scroll_terrain(&g);
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

                g.update_highscore();
            }

            mvprintw(
                2 * LINES() / 3,
                COLS() / 2 - 23,
                "PRESS 'JUMP' TO START AGAIN, 'QUIT' TO QUIT",
            );

            nodelay(stdscr(), false);
            update_highscore_file(&g);

            let key = getch();

            if key == KEY_QUIT {
                return;
            } else if key == KEY_JUMP {
                Game::run(g.highscore - 1);
                return;
            }
        }
    }

    fn update_highscore(&mut self) {
        if self.score > self.highscore {
            self.highscore = self.score;
        }
    }
}

pub fn get_highscore() -> u32 {
    let p: &str = &shellexpand::tilde(SAVE_FILE_PATH).to_string();

    let h: String = fs::read_to_string(p).unwrap_or_else(|e| {
        if e.kind() == ErrorKind::NotFound {
            create_highscore_file(p);
        } else {
            mvprintw(0, 0, &format!("Error reading the save file: {}\n", e));
            addstr("Press any key to continue.");
            getch();
        }

        "0".to_string()
    });

    if h.is_empty() {
        return 0;
    }

    h.parse::<u32>().unwrap_or_else(|e| {
        mvprintw(1, 0, &format!("Error parsing the save file: {}\n", e));
        addstr("The current value will be overwritten. Press any key to continue.");
        getch();

        0
    })
}

fn create_highscore_file(path: &str) {
    let f = fs::File::create(path);

    match f {
        Ok(mut file) => {
            file.write(b"0").unwrap();
            ()
        }
        Err(e) => {
            mvprintw(0, 0, &format!("Error creating the save file: {}\n", e));
            addstr("Press any key to continue.");
            getch();
        }
    };
}

pub fn update_highscore_file(g: &Game) {
    if g.score >= g.highscore {
        let p: &str = &shellexpand::tilde(SAVE_FILE_PATH).to_string();
        fs::write(p, (g.highscore - 1).to_string()).unwrap();
    }
}
