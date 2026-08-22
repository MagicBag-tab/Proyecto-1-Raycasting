# Liminal Maze 🌲🏜️❄️

Un motor de Raycasting en 3D escrito en Rust desde cero para el curso de Gráficas por Computadora.

## Características Implementadas (Rúbrica de 100 puntos)
- **Motor Base:** Renderizado de paredes en 3D a través de raymarching, movimiento con teclado, rotación de cámara con **Mouse**.
- **Texturizado:** Carga dinámica de imágenes con texturas distintas para 3 niveles diferentes. Texturizado correcto de paredes, cielo y **Piso** (Floor casting con perspectiva 3D). Soporte de tecla **T** para alternar con colores sólidos dinámicos por nivel. La textura del bloque final ahora usa la imagen `meta.png`.
- **Minimapa y UI:** Minimapa 2D superpuesto en la esquina superior derecha y un contador de **FPS** en tiempo real. 
- **Múltiples Niveles:** Pantalla de bienvenida interactiva donde se puede presionar 1, 2 o 3 para cargar diferentes mapas (`maze.txt`, `maze2.txt`, `maze3.txt`), alterando paletas de color y texturas.
- **Audio de Fondo:** Reproducción de música en hilo secundario mediante el crate `rodio` (`Post-Dream.mp3`).
- **Estados de Juego:** HUD, textos emergentes en amarillo (`font8x8`), y transición fluida a la pantalla de victoria usando imágenes importadas (`bienvenida.png`, `felicitaciones.png`).

## Instrucciones
- Ejecuta `cargo run --release` para un rendimiento óptimo.
- **W, A, S, D:** Moverse
- **Mouse (Horizontal):** Rotar cámara
- **T:** Alternar entre Modo Texturas y Modo Color Sólido
- **M:** Alternar la música (Versión normal / Taylor's Version 8-bits)
- **1, 2, 3:** Seleccionar nivel en la pantalla inicial
