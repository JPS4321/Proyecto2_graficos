mod framebuffer;
mod ray_intersect;
mod cube;
mod color;
mod camera;
mod light;
mod material;
mod texture;

use minifb::{ Window, WindowOptions, Key };
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
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

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
    objects: &[Cube], // Cambiado a cubos
    light: &Light,
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

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();

    let shadow_intensity = 0.0; // Simplificado, puedes agregar sombra más adelante
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect_vec(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
    }

    diffuse + reflect_color * reflectivity
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            framebuffer.set_current_color(pixel_color.to_hex());
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
        Color::new(255, 255, 255),  // Color base (puede ignorarse si tienes textura)
        50.0,
        [0.6, 0.3, 0.0, 0.0],
        1.0,
        Some(load_texture("./texture/grass.png")),  // Cargar la textura para todas las caras
        None,
        None,
    );

    // Material con textura diferente para el cubo central
    let center_cube_material = Material::new(
        Color::new(255, 255, 255),
        50.0,
        [0.6, 0.3, 0.0, 0.0],
        1.0,
        Some(load_texture("./texture/water.jpeg")),  // Textura distinta para los cubos centrales
        None,
        None,
    );

    // Crear una cuadrícula de 5x5 cubos de tamaño 0.5
    let mut objects = Vec::new();
    let cube_size = 0.5;
    let low_cube_height = 0.25;
    let grid_size = 5;

    for i in 0..grid_size {
        for j in 0..grid_size {
            let x_pos = i as f32 * cube_size - (grid_size as f32 * cube_size) / 2.0; // Ajustar la posición X
            let z_pos = j as f32 * cube_size - (grid_size as f32 * cube_size) / 2.0; // Ajustar la posición Z

            // Cubos centrales más bajos
            if i >= 2 && i <= 3 && j >= 2 && j <= 3 {
                objects.push(Cube {
                    min_corner: Vec3::new(x_pos, 0.0, z_pos),
                    max_corner: Vec3::new(x_pos + cube_size, low_cube_height, z_pos + cube_size),
                    material: textured_material.clone(),
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
                min_corner: Vec3::new(x_pos, low_cube_height, z_pos),
                max_corner: Vec3::new(x_pos + cube_size, cube_size, z_pos + cube_size),
                material: center_cube_material.clone(),
            });
        }
    }

    let mut camera = Camera::new(
        Vec3::new(0.0, 1.5, 3.0),  // Cámara un poco más arriba para visualizar la cuadrícula
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let light = Light::new(
        Vec3::new(0.0, 5.0, 5.0),  // Luz desde arriba y hacia adelante
        Color::new(255, 255, 255), // Color blanco
        2.0                         // Aumentar la intensidad de la luz
    );

    let rotation_speed = PI / 10.0;
    let move_speed = 0.1;  // Velocidad de acercarse/alejarse

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Rotación de la cámara como antes
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

        // Acercarse con la tecla 'W'
        if window.is_key_down(Key::W) {
            let direction = (camera.center - camera.eye).normalize(); // Dirección hacia el centro
            camera.eye += direction * move_speed; // Mover la cámara hacia el centro
        }

        // Alejarse con la tecla 'S'
        if window.is_key_down(Key::S) {
            let direction = (camera.center - camera.eye).normalize(); // Dirección hacia el centro
            camera.eye -= direction * move_speed; // Mover la cámara hacia atrás
        }

        render(&mut framebuffer, &objects, &camera, &light);
        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
        std::thread::sleep(frame_delay);
    }
}
