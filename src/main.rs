mod caster;
mod framebuffer;
mod maze;
mod player;

use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::Duration;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, Maze};
use crate::player::{process_events, Player};

const BLOCK_SIZE: usize = 15;

const NUM_RAYS: usize = 5;

const FOV: f32 = PI / 3.0;

fn cell_color(cell: char) -> u32 {
    match cell {
        '+' => 0x00AAFF, // columnas
        '-' => 0xFF5555, // paredes horizontales
        '|' => 0xFF5555, // paredes verticales
        'g' | 'G' => 0x00FF00, // meta
        _ => 0xFFDDDD,   // cualquier otra cosa
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, cell: char) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(cell_color(cell));

    for x in xo..xo + BLOCK_SIZE {
        for y in yo..yo + BLOCK_SIZE {
            framebuffer.point(x, y);
        }
    }
}

fn render(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let d_proj = hw / (FOV / 2.0).tan();

    for i in 0..num_rays {
        let ray_fraction = i as f32 / num_rays as f32; // de 0.0 a 1.0
        let angle = player.a - FOV / 2.0 + FOV * ray_fraction;
        let intersect = cast_ray(maze, player, angle, BLOCK_SIZE);
        
        let mut distance = intersect.distance * (angle - player.a).cos();
        if distance < 1.0 {
            distance = 1.0;
        }
        
        if intersect.impact != ' ' {
            let color = cell_color(intersect.impact);
            
            let wall_height = (BLOCK_SIZE as f32 * d_proj / distance) as usize;
            
            let start_y = if wall_height > framebuffer.height {
                0
            } else {
                (framebuffer.height - wall_height) / 2
            };
            
            let end_y = (start_y + wall_height).min(framebuffer.height);
            
            // Draw ceiling
            framebuffer.set_current_color(0x333355);
            for y in 0..start_y {
                framebuffer.point(i, y);
            }
            
            // Draw wall estaca
            framebuffer.set_current_color(color);
            for y in start_y..end_y {
                framebuffer.point(i, y);
            }
            
            // Draw floor
            framebuffer.set_current_color(0x555555);
            for y in end_y..framebuffer.height {
                framebuffer.point(i, y);
            }
        } else {
            // Draw just ceiling and floor if nothing hit
            framebuffer.set_current_color(0x333355);
            for y in 0..(framebuffer.height / 2) {
                framebuffer.point(i, y);
            }
            framebuffer.set_current_color(0x555555);
            for y in (framebuffer.height / 2)..framebuffer.height {
                framebuffer.point(i, y);
            }
        }
    }
}

fn main() {
    let window_width = 1024;
    let window_height = 768;
    let framebuffer_width = 1024;
    let framebuffer_height = 768;
    let frame_delay = Duration::from_millis(16);

    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new(
        "Maze Runner",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        process_events(&window, &mut player, &maze, BLOCK_SIZE);

        let i = player.pos.x as usize / BLOCK_SIZE;
        let j = player.pos.y as usize / BLOCK_SIZE;
        if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

        framebuffer.clear();

        render(&mut framebuffer, &maze, &player);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}