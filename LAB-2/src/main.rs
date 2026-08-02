mod framebuffer;

use crate::framebuffer::Framebuffer;
use minifb::{Key, Window, WindowOptions, Scale};

// El tablero: cada celda es un pixel. Baja resolución a propósito para que se vea bien.
const WIDTH: usize = 100;
const HEIGHT: usize = 100;

const ALIVE: u32 = 0xFFFFFF; // blanco = célula viva
const DEAD: u32 = 0x000000;  // negro  = célula muerta


// Pinta una lista de celdas relativas a un origen (x0, y0).
fn stamp(fb: &mut Framebuffer, x0: usize, y0: usize, cells: &[(usize, usize)]) {
    fb.set_current_color(ALIVE);
    for (dx, dy) in cells {
        fb.point((x0 + dx) % WIDTH, (y0 + dy) % HEIGHT);
    }
}

// ---------- Un organismo por función ----------

// Blinker: oscilador de 3 celdas, parpadea horizontal <-> vertical.
fn blinker(fb: &mut Framebuffer, x: usize, y: usize) {
    stamp(fb, x, y, &[(0, 0), (1, 0), (2, 0)]);
}

// Glider: la nave más pequeña, camina en diagonal hacia abajo-derecha.
fn glider(fb: &mut Framebuffer, x: usize, y: usize) {
    stamp(fb, x, y, &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
}

// LWSS (lightweight spaceship): nave ligera que viaja hacia la derecha.
fn lwss(fb: &mut Framebuffer, x: usize, y: usize) {
    stamp(fb, x, y, &[
        (0, 0), (3, 0),
        (4, 1),
        (0, 2), (4, 2),
        (1, 3), (2, 3), (3, 3), (4, 3),
    ]);
}

// Pulsar: oscilador grande de periodo 3, muy vistoso (13x13).
fn pulsar(fb: &mut Framebuffer, x: usize, y: usize) {
    stamp(fb, x, y, &[
        (2,0),(3,0),(4,0),(8,0),(9,0),(10,0),
        (0,2),(5,2),(7,2),(12,2),
        (0,3),(5,3),(7,3),(12,3),
        (0,4),(5,4),(7,4),(12,4),
        (2,5),(3,5),(4,5),(8,5),(9,5),(10,5),
        (2,7),(3,7),(4,7),(8,7),(9,7),(10,7),
        (0,8),(5,8),(7,8),(12,8),
        (0,9),(5,9),(7,9),(12,9),
        (0,10),(5,10),(7,10),(12,10),
        (2,12),(3,12),(4,12),(8,12),(9,12),(10,12),
    ]);
}

// Gosper Glider Gun: el cañón que dispara un glider nuevo cada 30 turnos, para siempre.
fn gosper_gun(fb: &mut Framebuffer, x: usize, y: usize) {
    stamp(fb, x, y, &[
        (0,4),(0,5),(1,4),(1,5),
        (10,4),(10,5),(10,6),
        (11,3),(11,7),
        (12,2),(12,8),
        (13,2),(13,8),
        (14,5),
        (15,3),(15,7),
        (16,4),(16,5),(16,6),
        (17,5),
        (20,2),(20,3),(20,4),
        (21,2),(21,3),(21,4),
        (22,1),(22,5),
        (24,0),(24,1),(24,5),(24,6),
        (34,2),(34,3),(35,2),(35,3),
    ]);
}

//Creación de los organismos 
fn seed_initial_pattern(fb: &mut Framebuffer) {
    // Cañones: la lluvia de gliders que llena el tablero.
    gosper_gun(fb, 2, 2);
    gosper_gun(fb, 52, 2);
    gosper_gun(fb, 2, 50);
    gosper_gun(fb, 52, 50);

    // Pulsares: adornos que laten.
    pulsar(fb, 30, 72);
    pulsar(fb, 62, 72);
    pulsar(fb, 80, 40);

    // Naves ligeras: viajan a la derecha.
    lwss(fb, 6, 88);
    lwss(fb, 30, 92);
    lwss(fb, 60, 92);

    // Flotilla de gliders.
    glider(fb, 10, 70);
    glider(fb, 18, 66);
    glider(fb, 86, 60);
    glider(fb, 90, 80);
    glider(fb, 75, 88);

    // Acentos que parpadean.
    blinker(fb, 45, 45);
    blinker(fb, 92, 10);
    blinker(fb, 8, 40);
}

// Cuenta cuántos de los 8 vecinos de (x, y) están vivos.
// Tablero toroidal: 
fn count_alive_neighbors(old: &[bool], x: usize, y: usize) -> usize {
    let mut count = 0;
    for dy in [-1i32, 0, 1] {
        for dx in [-1i32, 0, 1] {
            if dx == 0 && dy == 0 {
                continue; // no me cuento a mí mismo como vecino
            }
            let nx = (x as i32 + dx).rem_euclid(WIDTH as i32) as usize;
            let ny = (y as i32 + dy).rem_euclid(HEIGHT as i32) as usize;
            if old[ny * WIDTH + nx] {
                count += 1;
            }
        }
    }
    count
}

// Esta es la función "render" en la que se implementa toda la lógica del juego 
fn render(framebuffer: &mut Framebuffer) {
    // FASE 1: la foto del turno anterior. Leemos TODO el tablero a un arreglo de bool.
    let mut old = vec![false; WIDTH * HEIGHT];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            old[y * WIDTH + x] = framebuffer.get_color(x, y) == ALIVE;
        }
    }

    // FASE 2 y 3: decidir mirando la foto vieja, y pintar el resultado.
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let alive = old[y * WIDTH + x];
            let neighbors = count_alive_neighbors(&old, x, y);

            // Las cuatro reglas, condensadas:
            //   viva con 2 o 3 vecinos   -> sobrevive
            //   muerta con exactamente 3 -> nace
            //   cualquier otro caso      -> muere / sigue muerta
            let next_alive = matches!(
                (alive, neighbors),
                (true, 2) | (true, 3) | (false, 3)
            );

            framebuffer.set_current_color(if next_alive { ALIVE } else { DEAD });
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    // El framebuffer chiquito es nuestro tablero de juego (100x100 células).
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(DEAD);
    framebuffer.clear();

    // Patrón inicial creativo.
    seed_initial_pattern(&mut framebuffer);

    // La ventana. Scale::X8 estira el tablero de 100x100 a 800x800 en pantalla.
    let mut window = Window::new(
        "Conway's Game of Life - Julio",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        },
    )
    .expect("No se pudo crear la ventana");

    // ~15 turnos por segundo. Cada frame es un turno del juego.
    window.set_target_fps(15);

    // Loop de render en tiempo real.
    while window.is_open() && !window.is_key_down(Key::Escape) {
        render(&mut framebuffer);

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .expect("Error al actualizar la ventana");
    }
}