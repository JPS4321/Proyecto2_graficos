mod framebuffer;
mod ray_intersect;
mod cube;
mod color;
mod camera;
mod light;
mod material;
mod texture;

use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;

use nalgebra_glm::reflect_vec;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::texture::load_texture;

const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(0.27, 0.56, 0.89); // Color del cielo (valores entre 0.0 y 1.0)

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    lights: &[Light],  // Cambiamos a una lista de luces
    depth: u32,
) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return SKYBOX_COLOR;
    }

    let mut final_color = Color::black();  // Color resultante acumulado

    // Iterar sobre cada fuente de luz
    for light in lights {
        let light_dir = (light.position - intersect.point).normalize();
        let view_dir = (ray_origin - intersect.point).normalize();

        // Cálculo de la intensidad difusa
        let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
        let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light.intensity;

        // Reflexión (si es aplicable)
        let mut reflect_color = Color::black();
        let reflectivity = intersect.material.albedo[2];
        if reflectivity > 0.0 {
            let reflect_dir = reflect_vec(&ray_direction, &intersect.normal).normalize();
            let reflect_origin = offset_origin(&intersect, &reflect_dir);
            reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, lights, depth + 1);
        }

        // Acumular el color final con la contribución de cada luz
        final_color = final_color + diffuse + reflect_color * reflectivity;
    }

    final_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, lights: &[Light]) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = std::f32::consts::PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, lights, 0);

            framebuffer.set_current_color(pixel_color.to_u32());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Refractor",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // Material con textura para todas las caras del cubo
    let textured_material = Material::new(
        Color::new(1.0, 1.0, 1.0),  // Color base en formato f32
        50.0,
        [0.6, 0.3, 0.0, 0.0],
        1.0,
        Some(load_texture("./texture/grass.png")),
        None,
        None,
    );

    // Material con textura diferente para el cubo central
    let center_cube_material = Material::new(
        Color::new(1.0, 1.0, 1.0),
        50.0,
        [0.6, 0.3, 0.0, 0.0],
        1.0,
        Some(load_texture("./texture/water.jpeg")),
        None,
        None,
    );

    // Material para la mini torre con textura de piedra
    let tower_material = Material::new(
        Color::new(1.0, 1.0, 1.0),  // Color base, pero la textura lo sobreescribe
        50.0,  // Valor especular
        [0.6, 0.3, 0.0, 0.0],  // Albedo
        1.0,  // Índice de refracción
        Some(load_texture("./texture/cobble.png")),  // Asignar la textura que cargaste
        None,  // Sin mapa de normales
        None,  // Sin emisión
    );

    // Crear una cuadrícula de 5x5 cubos de tamaño 0.5
    let mut objects = Vec::new();
    let cube_size = 0.5;
    let low_cube_height = 0.25;
    let grid_size = 5;

    for i in 0..grid_size {
        for j in 0..grid_size {
            let x_pos = i as f32 * cube_size - (grid_size as f32 * cube_size) / 2.0;
            let z_pos = j as f32 * cube_size - (grid_size as f32 * cube_size) / 2.0;

            // Cubos centrales más bajos (reemplaza los cubos normales)
            if i >= 2 && i <= 3 && j >= 2 && j <= 3 {
                objects.push(Cube {
                    min_corner: Vec3::new(x_pos, 0.0, z_pos),
                    max_corner: Vec3::new(x_pos + cube_size, low_cube_height, z_pos + cube_size),
                    material: center_cube_material.clone(),
                });
            } else {
                // Otros cubos normales
                objects.push(Cube {
                    min_corner: Vec3::new(x_pos, 0.0, z_pos),
                    max_corner: Vec3::new(x_pos + cube_size, cube_size, z_pos + cube_size),
                    material: textured_material.clone(),
                });
            }
        }
    }

    // Añadir cuatro cubos más pequeños en el área central
    for i in 2..4 {
        for j in 2..4 {
            let x_pos = i as f32 * cube_size - (grid_size as f32 * cube_size) / 2.0;
            let z_pos = j as f32 * cube_size - (grid_size as f32 * cube_size) / 2.0;

            objects.push(Cube {
                min_corner: Vec3::new(x_pos, low_cube_height, z_pos),  // Altura más baja
                max_corner: Vec3::new(x_pos + cube_size, cube_size - 0.03, z_pos + cube_size),  // Más pequeños
                material: center_cube_material.clone(),
            });
        }
    }

    // Posicionar la torre en la esquina superior izquierda de la cuadrícula
    let x_pos = 0.0 * cube_size - (grid_size as f32 * cube_size) / 2.0;
    let z_pos = 0.0 * cube_size - (grid_size as f32 * cube_size) / 2.0;

    // Primer bloque de la torre (altura 0.5)
    objects.push(Cube {
        min_corner: Vec3::new(x_pos, cube_size, z_pos),  // Encima del cubo base
        max_corner: Vec3::new(x_pos + cube_size, cube_size * 1.5, z_pos + cube_size),
        material: tower_material.clone(),  // Usar la textura de piedra
    });

    // Segundo bloque de la torre (altura 0.5)
    objects.push(Cube {
        min_corner: Vec3::new(x_pos, cube_size * 1.5, z_pos),  // Encima del primer bloque
        max_corner: Vec3::new(x_pos + cube_size, cube_size * 2.0, z_pos + cube_size),
        material: tower_material.clone(),  // Usar la textura de piedra
    });

    let mut camera = Camera::new(
        Vec3::new(0.0, 1.5, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Añadir dos luces
    let light1 = Light::new(
        Vec3::new(0.0, 5.0, 5.0),  // Primera luz (como ya existe)
        Color::new(1.0, 1.0, 1.0),  // Luz blanca
        2.0,  // Intensidad
    );

    let light2 = Light::new(
        Vec3::new(0.0, 5.0, -5.0),  // Segunda luz en el lado opuesto
        Color::new(1.0, 0.5, 0.5),  // Luz con un tono rojizo
        1.5,  // Intensidad un poco menor
    );

    // Guardamos ambas luces en un vector
    let lights = vec![light1, light2];

    let rotation_speed = PI / 10.0;
    let move_speed = 0.1;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }

        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }

        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }

        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        if window.is_key_down(Key::W) {
            let direction = (camera.center - camera.eye).normalize();
            camera.eye += direction * move_speed;
        }

        if window.is_key_down(Key::S) {
            let direction = (camera.center - camera.eye).normalize();
            camera.eye -= direction * move_speed;
        }

        render(&mut framebuffer, &objects, &camera, &lights);
        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
        std::thread::sleep(frame_delay);
    }
}
