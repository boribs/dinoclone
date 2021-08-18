use crate::*;

pub fn autoplay(player: &mut p::Player, terrain: &t::Terrain) {
    let a = terrain.vec[(PX + 3) as usize].obstacle;
    let b = terrain.vec[(PX + 8) as usize].obstacle;

    if a && !b {
        player.jump(terrain);
    }
}
