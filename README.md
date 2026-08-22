# Liminal Maze 🌲🏜️❄️

Un motor de Raycasting en 3D escrito en Rust desde cero para el curso de Gráficas por Computadora.

## Video de Prueba
<!-- Reproduce el video de demostración desde la carpeta assets -->
<video src="./assets/demo_proyecto.mp4" controls="controls" width="100%">
  Tu navegador no soporta el elemento de video.
</video>

## Características Implementadas (Rúbrica de 100 puntos)
- **Motor Base:** Renderizado de paredes en 3D a través de raymarching, movimiento con teclado, rotación de cámara con **Mouse**.
- **Texturizado y Parallax:** Carga de imágenes con texturas distintas para 3 niveles diferentes (Atardecer, Noche, Amanecer). Texturizado correcto de paredes, cielo y **Piso** (Floor casting con perspectiva 3D). Además, se implementó un sistema de **Nubes en Parallax Horizontal** con transparencias sobrepuestas al cielo para dar profundidad infinita. Soporte de tecla **T** para alternar con colores sólidos. La textura del bloque final ahora usa la imagen `meta.png`.
- **Sprites 3D:** Renderizado de sprites 3D tipo billboard con ordenamiento de Z-buffer para decorar el mapa, que escalan correctamente con la distancia.
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
