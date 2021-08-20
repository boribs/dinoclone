use crate::*;

pub fn autoplay(player: &mut p::Player, terrain: &t::Terrain, game: &Game) {
    let a: bool = {
        let half_max_air_time = (game.max_air_time >> 1) + 1;
        let i = (PX + half_max_air_time) as usize;

        let mut near = false;
        for j in 0..half_max_air_time as usize + 1 {
            if terrain.vec[i - j].obstacle {
                near = true;
                break;
            }
        }

        near
    };
    let b = terrain.vec[(PX + game.max_air_time) as usize].obstacle;

    if a && !b && terrain.vec[PX as usize].unit_type == t::TerrainType::Flat {
        player.jump(terrain);
    }
}
