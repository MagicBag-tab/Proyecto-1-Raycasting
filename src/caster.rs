use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub x: f32,
    pub y: f32,
}

pub fn cast_ray(
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
) -> Intersect {
    let mut d = 0.0;

    loop {
        let exact_x = player.pos.x + d * a.cos();
        let exact_y = player.pos.y + d * a.sin();
        let x = exact_x as usize;
        let y = exact_y as usize;

        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return Intersect { distance: d, impact: ' ', x: exact_x, y: exact_y };
        }

        let cell = maze[j][i];
        if cell != ' ' {
            return Intersect { distance: d, impact: cell, x: exact_x, y: exact_y };
        }

        d += 0.1;
    }
}