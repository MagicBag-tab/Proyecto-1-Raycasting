use crate::maze::Maze;
use minifb::{Key, Window, MouseMode};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub last_mouse_x: Option<f32>,
}

pub fn process_events(window: &Window, player: &mut Player, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 4.0;
    const ROTATION_SPEED: f32 = PI / 40.0;
    const MOUSE_SENSITIVITY: f32 = 0.005;

    if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }
    
    if let Some((mouse_x, _mouse_y)) = window.get_mouse_pos(MouseMode::Pass) {
        if let Some(last_x) = player.last_mouse_x {
            let delta_x = mouse_x - last_x;
            player.a += delta_x * MOUSE_SENSITIVITY;
        }
        player.last_mouse_x = Some(mouse_x);
    }

    let mut new_x = player.pos.x;
    let mut new_y = player.pos.y;

    if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
        new_x += MOVE_SPEED * player.a.cos();
        new_y += MOVE_SPEED * player.a.sin();
    }

    if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
        new_x -= MOVE_SPEED * player.a.cos();
        new_y -= MOVE_SPEED * player.a.sin();
    }

    let i_x = new_x as usize / block_size;
    let j_y = player.pos.y as usize / block_size;
    if j_y < maze.len() && i_x < maze[j_y].len() {
        let cell = maze[j_y][i_x];
        if cell == ' ' || cell == 'g' || cell == 'G' || cell == 'p' {
            player.pos.x = new_x;
        }
    }

    let i_x_new = player.pos.x as usize / block_size;
    let j_y_new = new_y as usize / block_size;
    if j_y_new < maze.len() && i_x_new < maze[j_y_new].len() {
        let cell = maze[j_y_new][i_x_new];
        if cell == ' ' || cell == 'g' || cell == 'G' || cell == 'p' {
            player.pos.y = new_y;
        }
    }
}