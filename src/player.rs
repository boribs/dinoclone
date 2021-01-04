use ncurses::mvaddch;

use super::terrain as t;

const PLAYER_CHAR: u32 = '$' as u32;
const JUMP_TO_MAX_HEIGHT_DIST: i32 = 3;

#[derive(Copy, Clone, PartialEq)]
pub enum PlayerState {
    Idle,
    Running,
    Jumping,
    MaxHeight,
    Falling,
    Dead,
}

pub struct Player {
    pub y_pos: i32,
    pub air_dist: i32,
    pub state: PlayerState,
    pub remember_jump: bool,
}

impl Player {
    pub fn jump(&mut self, t: t::TerrainType) {
        if t == t::TerrainType::Up {
            self.remember_jump = true;
        } else if self.state == PlayerState::Running {
            self.state = PlayerState::Jumping;
        }
    }

    pub fn update_pos(
        &mut self,
        iy: i32,
        current_unit: &t::TerrainUnit,
        offset_y: i32,
        roffset_y: i32,
        max_air_time: i32,
    ) {
        match self.state {
            PlayerState::Jumping => {
                self.y_pos -= 1;
                self.air_dist += 1;

                if self.air_dist == JUMP_TO_MAX_HEIGHT_DIST {
                    self.state = PlayerState::MaxHeight;
                }

                if self.y_pos >= iy - roffset_y {
                    self.state = PlayerState::Running;
                }
            }
            PlayerState::MaxHeight => {
                if self.y_pos >= iy - roffset_y {
                    self.state = PlayerState::Running;
                    self.air_dist = 0;
                } else {
                    self.air_dist += 1;

                    if self.air_dist == max_air_time {
                        self.state = PlayerState::Falling;
                    }
                }
            }
            PlayerState::Falling => {
                if self.y_pos >= iy - roffset_y {
                    self.state = PlayerState::Running;
                    self.air_dist = 0;
                } else {
                    self.y_pos += 1;
                }
            }
            _ => {
                if self.remember_jump && current_unit.unit_type != t::TerrainType::Up {
                    self.state = PlayerState::Jumping;
                    self.remember_jump = false;
                } else {
                    self.y_pos = iy - roffset_y
                }
            }
        };

        if self.y_pos == current_unit.initial_y + offset_y && current_unit.obstacle {
            self.state = PlayerState::Dead;
        }
    }
}

pub fn draw_player(p: &Player, px: i32) {
    mvaddch(p.y_pos, px, PLAYER_CHAR);
}
