use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

pub fn cast_ray(
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
) -> Intersect {
    let mut d = 0.0;

    loop {
        let x = (player.pos.x + d * a.cos()) as usize;
        let y = (player.pos.y + d * a.sin()) as usize;

        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return Intersect { distance: d, impact: ' ' };
        }

        let cell = maze[j][i];
        if cell != ' ' {
            return Intersect { distance: d, impact: cell };
        }

        d += 0.1;
    }
}