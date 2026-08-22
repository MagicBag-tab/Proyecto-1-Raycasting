mod caster;
mod framebuffer;
mod maze;
mod player;
mod texture;

use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::Duration;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, Maze};
use crate::player::{process_events, Player};
use crate::texture::Texture;

const BLOCK_SIZE: usize = 15;

const FOV: f32 = PI / 3.0;

fn cell_color(cell: char) -> u32 {
    match cell {
        '+' | '-' | '|' => 0x228B22, // Verde bosque para que haga match con los arbustos
        'g' | 'G' => 0x00FF00, // Meta brillante
        _ => 0x228B22,   // Cualquier otra pared
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    let minimap_block_size = 4; 
    let offset_x = framebuffer.width - (maze[0].len() * minimap_block_size) - 20; 
    let offset_y = 20; 

    let map_width = maze[0].len() * minimap_block_size;
    let map_height = maze.len() * minimap_block_size;
    
    framebuffer.set_current_color(0xFFFFFF); 
    for x in (offset_x - 2)..(offset_x + map_width + 2) {
        for y in (offset_y - 2)..(offset_y + map_height + 2) {
            framebuffer.point(x, y);
        }
    }
    
    framebuffer.set_current_color(0x111122); 
    for x in offset_x..offset_x + map_width {
        for y in offset_y..offset_y + map_height {
            framebuffer.point(x, y);
        }
    }

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

    framebuffer.set_current_color(0xFFFFFF); 
    let player_px = offset_x + (player.pos.x / BLOCK_SIZE as f32 * minimap_block_size as f32) as usize;
    let player_py = offset_y + (player.pos.y / BLOCK_SIZE as f32 * minimap_block_size as f32) as usize;
    
    for x in player_px.saturating_sub(2)..=player_px + 2 {
        for y in player_py.saturating_sub(2)..=player_py + 2 {
            framebuffer.point(x, y);
        }
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    wall_texture: &Texture,
    sky_texture: &Texture,
    floor_texture: &Texture,
) {
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let d_proj = hw / (FOV / 2.0).tan();

    for i in 0..num_rays {
        let screen_x = i as f32 - hw;
        
        let ray_angle_relative = (screen_x / d_proj).atan();
        let angle = player.a + ray_angle_relative;
        
        let intersect = cast_ray(maze, player, angle, BLOCK_SIZE);
        
        let mut distance = intersect.distance * ray_angle_relative.cos();
        if distance < 1.0 {
            distance = 1.0;
        }
        
        if intersect.impact != ' ' {
            let wall_height = (BLOCK_SIZE as f32 * d_proj / distance) as usize;
            
            let start_y = if wall_height > framebuffer.height {
                0
            } else {
                (framebuffer.height - wall_height) / 2
            };
            
            let end_y = (start_y + wall_height).min(framebuffer.height);
            
            if player.use_textures {
                for y in 0..start_y {
                    let sky_x = (i as u32) % sky_texture.width;
                    let sky_y = (y as u32) % sky_texture.height;
                    framebuffer.set_current_color(sky_texture.get_pixel(sky_x, sky_y));
                    framebuffer.point(i, y);
                }
                
                // Draw wall (texture mapped by distance and impact)
                // Calculate texture X coordinate based on where the ray hit the block
                let hit_x = intersect.x - (intersect.x / BLOCK_SIZE as f32).floor() * BLOCK_SIZE as f32;
                let hit_y = intersect.y - (intersect.y / BLOCK_SIZE as f32).floor() * BLOCK_SIZE as f32;
                
                // Para que no se vea gigante la pared, podemos "repetir" la textura (tiling) multiplicando
                let wall_tile_factor = 2.0; 
                
                let mut tex_x = if hit_x < 0.1 || hit_x > BLOCK_SIZE as f32 - 0.1 {
                    (hit_y / BLOCK_SIZE as f32 * wall_texture.width as f32 * wall_tile_factor) as u32
                } else {
                    (hit_x / BLOCK_SIZE as f32 * wall_texture.width as f32 * wall_tile_factor) as u32
                };
                tex_x = tex_x % wall_texture.width;
                
                for y in start_y..end_y {
                    let tex_y = if wall_height > framebuffer.height {
                        let top_offset = (wall_height - framebuffer.height) / 2;
                        let adjusted_y = y + top_offset;
                        (adjusted_y as f32 / wall_height as f32 * wall_texture.height as f32 * wall_tile_factor) as u32
                    } else {
                        let adjusted_y = y - start_y;
                        (adjusted_y as f32 / wall_height as f32 * wall_texture.height as f32 * wall_tile_factor) as u32
                    };
                    let tex_y = tex_y % wall_texture.height;
                    
                    framebuffer.set_current_color(wall_texture.get_pixel(tex_x, tex_y));
                    framebuffer.point(i, y);
                }
                
                // Draw floor (proper 3D floor casting)
                let center_y = framebuffer.height as f32 / 2.0;
                for y in end_y..framebuffer.height {
                    let p = y as f32 - center_y;
                    // Prevenir división por 0
                    if p > 0.0 {
                        let perp_dist = (BLOCK_SIZE as f32 / 2.0) * d_proj / p;
                        let actual_dist = perp_dist / ray_angle_relative.cos();
                        
                        let floor_world_x = player.pos.x + actual_dist * angle.cos();
                        let floor_world_y = player.pos.y + actual_dist * angle.sin();
                        
                        let floor_x = (floor_world_x * (floor_texture.width as f32 / BLOCK_SIZE as f32)) as u32 % floor_texture.width;
                        let floor_y = (floor_world_y * (floor_texture.height as f32 / BLOCK_SIZE as f32)) as u32 % floor_texture.height;
                        
                        framebuffer.set_current_color(floor_texture.get_pixel(floor_x, floor_y));
                    }
                    framebuffer.point(i, y);
                }
            } else {
                // Modo color sólido
                framebuffer.set_current_color(0x191970); // Color del cielo
                for y in 0..start_y {
                    framebuffer.point(i, y);
                }
                
                framebuffer.set_current_color(cell_color(intersect.impact)); // Color de la pared
                for y in start_y..end_y {
                    framebuffer.point(i, y);
                }
                
                framebuffer.set_current_color(0x556B2F); // Color del piso
                for y in end_y..framebuffer.height {
                    framebuffer.point(i, y);
                }
            }
        } else {
            // Draw sky and floor for empty rays (no wall hit)
            if player.use_textures {
                for y in 0..(framebuffer.height / 2) {
                    let sky_x = (i as u32) % sky_texture.width;
                    let sky_y = (y as u32) % sky_texture.height;
                    framebuffer.set_current_color(sky_texture.get_pixel(sky_x, sky_y));
                    framebuffer.point(i, y);
                }
                
                let center_y = framebuffer.height as f32 / 2.0;
                for y in (framebuffer.height / 2)..framebuffer.height {
                    let p = y as f32 - center_y;
                    if p > 0.0 {
                        let perp_dist = (BLOCK_SIZE as f32 / 2.0) * d_proj / p;
                        let actual_dist = perp_dist / ray_angle_relative.cos();
                        let floor_world_x = player.pos.x + actual_dist * angle.cos();
                        let floor_world_y = player.pos.y + actual_dist * angle.sin();
                        let floor_x = (floor_world_x * (floor_texture.width as f32 / BLOCK_SIZE as f32)) as u32 % floor_texture.width;
                        let floor_y = (floor_world_y * (floor_texture.height as f32 / BLOCK_SIZE as f32)) as u32 % floor_texture.height;
                        framebuffer.set_current_color(floor_texture.get_pixel(floor_x, floor_y));
                    }
                    framebuffer.point(i, y);
                }
            } else {
                for y in 0..(framebuffer.height / 2) {
                    framebuffer.set_current_color(0x191970);
                    framebuffer.point(i, y);
                }
                for y in (framebuffer.height / 2)..framebuffer.height {
                    framebuffer.set_current_color(0x556B2F);
                    framebuffer.point(i, y);
                }
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
    
    let sky_tex = Texture::new("./assets/sky_l1.png");
    let wall_tex = Texture::new("./assets/wall_l1.png");
    let floor_tex = Texture::new("./assets/suelo_l1.png");

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

        render(&mut framebuffer, &maze, &player, &wall_tex, &sky_tex, &floor_tex);
        render_minimap(&mut framebuffer, &maze, &player);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}