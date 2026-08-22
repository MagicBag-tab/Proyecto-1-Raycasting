mod caster;
mod framebuffer;
mod maze;
mod player;
mod texture;

use minifb::{Key, Window, WindowOptions, MouseMode};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, DeviceSinkBuilder, Player as RodioPlayer};

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, Maze};
use crate::player::{process_events, Player};
use crate::texture::Texture;

fn draw_text(framebuffer: &mut Framebuffer, text: &str, start_x: usize, start_y: usize, scale: usize) {
    let mut x_offset = start_x;
    for c in text.chars() {
        if let Some(glyph) = BASIC_FONTS.get(c) {
            for (y, row) in glyph.iter().enumerate() {
                for x in 0..8 {
                    if (row & (1 << x)) != 0 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                let px = x_offset + x * scale + dx;
                                let py = start_y + y * scale + dy;
                                if px < framebuffer.width && py < framebuffer.height {
                                    framebuffer.point(px, py);
                                }
                            }
                        }
                    }
                }
            }
        }
        x_offset += 8 * scale + 2;
    }
}

fn draw_text_with_border(
    framebuffer: &mut Framebuffer,
    text: &str,
    start_x: usize,
    start_y: usize,
    scale: usize,
    text_color: u32,
    border_color: u32,
) {
    let offset = if scale > 2 { 3 } else { 1 };
    
    framebuffer.set_current_color(border_color);
    for dx in [-1isize, 0, 1] {
        for dy in [-1isize, 0, 1] {
            if dx == 0 && dy == 0 {
                continue;
            }
            let px = (start_x as isize + dx * offset).max(0) as usize;
            let py = (start_y as isize + dy * offset).max(0) as usize;
            draw_text(framebuffer, text, px, py, scale);
        }
    }
    
    framebuffer.set_current_color(text_color);
    draw_text(framebuffer, text, start_x, start_y, scale);
}

const BLOCK_SIZE: usize = 15;

const FOV: f32 = PI / 3.0;

fn cell_color(cell: char, level: u8) -> u32 {
    if cell == 'g' || cell == 'G' {
        return 0x00FF00; // Meta brillante
    }
    match level {
        2 => 0xD2B48C, // Arena/Ladrillo para el desierto
        3 => 0xB0E0E6, // Hielo/Azul claro para la nieve
        _ => 0x228B22, // Verde bosque para el nivel 1
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, current_level: u8) {
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
                let color = cell_color(cell, current_level);
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
    current_level: u8,
) {
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let d_proj = hw / (FOV / 2.0).tan();

    let sky_color = match current_level {
        2 => 0x87CEEB, // Cielo claro desierto
        3 => 0xA9A9A9, // Cielo gris nieve
        _ => 0x191970, // Noche/oscuro nivel 1
    };
    
    let floor_color = match current_level {
        2 => 0xDEB887, // Arena oscura
        3 => 0xFFFFFF, // Nieve blanca
        _ => 0x556B2F, // Pasto oscuro
    };

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
                framebuffer.set_current_color(sky_color); // Color del cielo
                for y in 0..start_y {
                    framebuffer.point(i, y);
                }
                
                framebuffer.set_current_color(cell_color(intersect.impact, current_level)); // Color de la pared
                for y in start_y..end_y {
                    framebuffer.point(i, y);
                }
                
                framebuffer.set_current_color(floor_color); // Color del piso
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
                    framebuffer.set_current_color(sky_color);
                    framebuffer.point(i, y);
                }
                for y in (framebuffer.height / 2)..framebuffer.height {
                    framebuffer.set_current_color(floor_color);
                    framebuffer.point(i, y);
                }
            }
        }
    }
}

enum GameState {
    Welcome,
    Playing,
    Success,
}

fn main() {
    let window_width = 1024;
    let window_height = 768;
    let framebuffer_width = 1024;
    let framebuffer_height = 768;
    let frame_delay = Duration::from_millis(16);

    // Audio inicialización
    let _sink_handle = DeviceSinkBuilder::open_default_sink().ok();
    let _player = _sink_handle.as_ref().map(|handle| RodioPlayer::connect_new(&handle.mixer()));
    if let Some(player) = &_player {
        if let Ok(file) = File::open("./assets/Post-Dream.mp3") {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                player.append(source);
            }
        }
    }

    let (mut maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);
    
    // Load default textures
    let mut sky_tex = Texture::new("./assets/sky_l1.png");
    let mut wall_tex = Texture::new("./assets/wall_l1.png");
    let mut floor_tex = Texture::new("./assets/suelo_l1.png");
    let welcome_bg = Texture::new("./assets/bienvenida.png");
    let success_bg = Texture::new("./assets/felicitaciones.png");

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new(
        "Liminal Maze",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let mut last_time = Instant::now();
    let mut fps = 0;
    
    let mut level_start_time = Instant::now();
    let mut game_state = GameState::Welcome;
    let mut current_level = 1u8;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(last_time).as_secs_f32();
        last_time = current_time;
        
        if frame_time > 0.0 {
            fps = (1.0 / frame_time) as u32;
        }

        framebuffer.clear();

        match game_state {
            GameState::Welcome => {
                // Dibujar imagen de fondo escalada a la pantalla
                for y in 0..framebuffer.height {
                    for x in 0..framebuffer.width {
                        let tex_x = (x as f32 / framebuffer.width as f32 * welcome_bg.width as f32) as u32;
                        let tex_y = (y as f32 / framebuffer.height as f32 * welcome_bg.height as f32) as u32;
                        framebuffer.set_current_color(welcome_bg.get_pixel(tex_x, tex_y));
                        framebuffer.point(x, y);
                    }
                }
                
                draw_text_with_border(&mut framebuffer, "LIMINAL MAZE", 200, 200, 6, 0xFFDD55, 0x000000);
                
                draw_text_with_border(&mut framebuffer, "Selecciona un Nivel para empezar:", 200, 350, 2, 0xFFDD55, 0x000000);
                draw_text_with_border(&mut framebuffer, "[1] Nivel 1 (Bosque)", 250, 420, 2, 0xFFDD55, 0x000000);
                draw_text_with_border(&mut framebuffer, "[2] Nivel 2 (Desierto/L2)", 250, 480, 2, 0xFFDD55, 0x000000);
                draw_text_with_border(&mut framebuffer, "[3] Nivel 3 (Nieve/L3)", 250, 540, 2, 0xFFDD55, 0x000000);
                
                let mut level_selected = 0;
                if window.is_key_down(Key::Key1) { level_selected = 1; }
                else if window.is_key_down(Key::Key2) { level_selected = 2; }
                else if window.is_key_down(Key::Key3) { level_selected = 3; }
                
                if level_selected > 0 {
                    current_level = level_selected;
                    sky_tex = Texture::new(&format!("./assets/sky_l{}.png", level_selected));
                    wall_tex = Texture::new(&format!("./assets/wall_l{}.png", level_selected));
                    floor_tex = Texture::new(&format!("./assets/suelo_l{}.png", level_selected));
                    
                    let maze_file = match level_selected {
                        2 => "./maze2.txt",
                        3 => "./maze3.txt",
                        _ => "./maze.txt",
                    };
                    
                    let (new_maze, new_player) = load_maze(maze_file, BLOCK_SIZE);
                    maze = new_maze;
                    player = new_player;
                    
                    level_start_time = Instant::now();
                    game_state = GameState::Playing;
                }
            },
            
            GameState::Playing => {
                process_events(&window, &mut player, &maze, BLOCK_SIZE);

                let i = player.pos.x as usize / BLOCK_SIZE;
                let j = player.pos.y as usize / BLOCK_SIZE;
                if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
                    game_state = GameState::Success;
                }

                render(&mut framebuffer, &maze, &player, &wall_tex, &sky_tex, &floor_tex, current_level);
                render_minimap(&mut framebuffer, &maze, &player, current_level);
                
                let level_name = match current_level {
                    2 => "Nivel 2: Desierto",
                    3 => "Nivel 3: Nieve",
                    _ => "Nivel 1: Bosque",
                };
                draw_text_with_border(&mut framebuffer, level_name, 20, 50, 2, 0xFFDD55, 0x000000);
                
                if level_start_time.elapsed().as_secs() < 2 {
                    draw_text_with_border(&mut framebuffer, "ENCUENTRA EL ARBOL PARA SALIR", 220, 350, 3, 0xFFDD55, 0x000000);
                }
            },
            
            GameState::Success => {
                // Dibujar imagen de felicitaciones escalada a la pantalla
                for y in 0..framebuffer.height {
                    for x in 0..framebuffer.width {
                        let tex_x = (x as f32 / framebuffer.width as f32 * success_bg.width as f32) as u32;
                        let tex_y = (y as f32 / framebuffer.height as f32 * success_bg.height as f32) as u32;
                        framebuffer.set_current_color(success_bg.get_pixel(tex_x, tex_y));
                        framebuffer.point(x, y);
                    }
                }
                
                draw_text_with_border(&mut framebuffer, "¡META ALCANZADA!", 150, 300, 6, 0xFFDD55, 0x000000);
                
                draw_text_with_border(&mut framebuffer, "Presiona ENTER para volver al menu", 200, 500, 2, 0xFFDD55, 0x000000);
                
                if window.is_key_down(Key::Enter) {
                    game_state = GameState::Welcome;
                }
            }
        }

        // Dibujar FPS siempre encima
        let fps_text = format!("FPS: {}", fps);
        draw_text_with_border(&mut framebuffer, &fps_text, 20, 20, 2, 0xFFDD55, 0x000000);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        // Controlar que los FPS no vayan más rápido de lo necesario
        let elapsed = current_time.elapsed();
        if elapsed < frame_delay {
            std::thread::sleep(frame_delay - elapsed);
        }
    }
}