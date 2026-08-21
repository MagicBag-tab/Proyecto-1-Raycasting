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

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    let minimap_block_size = 4; // Tamaño de cada bloque en el minimapa (pequeño para que quepa en la esquina)
    let offset_x = framebuffer.width - (maze[0].len() * minimap_block_size) - 20; // 20 px de margen derecho
    let offset_y = 20; // 20 px de margen superior

    // Dibujar fondo del minimapa
    let map_width = maze[0].len() * minimap_block_size;
    let map_height = maze.len() * minimap_block_size;
    
    // Dibujar borde del minimapa (2 px más grande)
    framebuffer.set_current_color(0xFFFFFF); // Borde blanco
    for x in (offset_x - 2)..(offset_x + map_width + 2) {
        for y in (offset_y - 2)..(offset_y + map_height + 2) {
            framebuffer.point(x, y);
        }
    }
    
    framebuffer.set_current_color(0x111122); // Un color oscuro azulado/grisáceo para el fondo
    for x in offset_x..offset_x + map_width {
        for y in offset_y..offset_y + map_height {
            framebuffer.point(x, y);
        }
    }

    // Dibujar el mapa
    for (row_idx, row) in maze.iter().enumerate() {
        for (col_idx, &cell) in row.iter().enumerate() {
            if cell != ' ' {
                let color = cell_color(cell);
                framebuffer.set_current_color(color);
                
                let px = offset_x + col_idx * minimap_block_size;
                let py = offset_y + row_idx * minimap_block_size;
                
                for x in px..(px + minimap_block_size) {
                    for y in py..(py + minimap_block_size) {
                        framebuffer.point(x, y);
                    }
                }
            }
        }
    }

    // Dibujar al jugador
    framebuffer.set_current_color(0xFFFFFF); // Blanco para el jugador
    let player_px = offset_x + (player.pos.x / BLOCK_SIZE as f32 * minimap_block_size as f32) as usize;
    let player_py = offset_y + (player.pos.y / BLOCK_SIZE as f32 * minimap_block_size as f32) as usize;
    
    for x in player_px.saturating_sub(2)..=player_px + 2 {
        for y in player_py.saturating_sub(2)..=player_py + 2 {
            framebuffer.point(x, y);
        }
    }
}

fn render(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    // x estacadas (rayos) dependiendo de la cantidad de columnas de la pantalla
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let d_proj = hw / (FOV / 2.0).tan();

    for i in 0..num_rays {
        // Calcular la posición x en el plano de proyección (-hw a +hw)
        let screen_x = i as f32 - hw;
        
        // Calcular el ángulo del rayo relativo al jugador usando trigonometría plana
        let ray_angle_relative = (screen_x / d_proj).atan();
        let angle = player.a + ray_angle_relative;
        
        let intersect = cast_ray(maze, player, angle, BLOCK_SIZE);
        
        // Corrección del ojo de pez perfecta: perpendicular al plano de la cámara
        let mut distance = intersect.distance * ray_angle_relative.cos();
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
        render_minimap(&mut framebuffer, &maze, &player);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}