extern crate num;
#[macro_use]
extern crate num_derive;
extern crate minifb;

mod entity;
mod geometry;
mod matter_tree;
mod space;
mod space_tree;
mod voxel_grid;

use entity::{Entity, EntityData};
use geometry::{Quadrant, Sphere, Vec3};
use space::{Space, SpaceConfiguration, SPACE_CELL_SIZE};
use space_tree::SpaceTree;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

struct Colors {
    space_node: u32,
    entity: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

fn draw_space_tree(colors: &Colors, buffer: &mut [u32], area: Rect, tree: &SpaceTree) {
    for y in 0..area.h {
        for x in 0..area.w {
            let offset = (area.y + y) * WIDTH + area.x + x;
            if x == 0 || y == 0 || x == area.w - 1 || y == area.h - 1 {
                buffer[offset] = colors.space_node;
            }
        }
    }

    for entity in tree.entities.iter() {
        let pos = entity.bounding_sphere.center;
        let x = (pos.x + SPACE_CELL_SIZE / 2) as usize * area.w / SPACE_CELL_SIZE as usize;
        let y = (pos.y + SPACE_CELL_SIZE / 2) as usize * area.h / SPACE_CELL_SIZE as usize;
        let offset = y * WIDTH + x;
        buffer[offset] = colors.space_node;
    }

    for (i, sub_tree) in tree.sub_trees.iter().enumerate() {
        if let Some(tree) = sub_tree {
            let quadrant: Quadrant = num::FromPrimitive::from_usize(i).unwrap();
            let mut sub_area = area;
            if quadrant.x_p() {
                sub_area.x += sub_area.w / 2;
            }
            if quadrant.y_p() {
                sub_area.y += sub_area.h / 2;
            }
            sub_area.w /= 2;
            sub_area.h /= 2;
            draw_space_tree(colors, buffer, sub_area, tree);
        }
    }
}

fn draw_space(colors: &Colors, buffer: &mut [u32], space: &Space) {
    // Wipe board
    for i in buffer.iter_mut() {
        *i = 0xFFFFFFFF;
    }

    draw_space_tree(
        colors,
        buffer,
        Rect {
            x: 0,
            y: 0,
            w: WIDTH,
            h: HEIGHT,
        },
        &space.universe,
    );
}

fn main() {
    let mut space = Space::new(SpaceConfiguration {
        tick_size: 10_000, // duration of a world tick in us

        // Optimization related
        gravity_threshold: 0, // cm/s^2
    });

    space.insert_entity(Box::new(Entity::new(
        Sphere {
            center: Vec3 { x: 0, y: 0, z: 0 },
            radius: 1,
        },
        EntityData::Creature,
    )));

    let colors = Colors {
        space_node: 0xFFFF0000,
        entity: 0xFF0000FF,
    };

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = minifb::Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        minifb::WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        draw_space(&colors, &mut buffer, &space);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        space.run();
    }
}
