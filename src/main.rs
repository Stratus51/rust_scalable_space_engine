extern crate num;
#[macro_use]
extern crate num_derive;
extern crate minifb;

mod entity;
mod geometry;
mod matter_tree;
mod player;
mod space;
mod space_tree;
mod voxel_grid;

use entity::{Entity, EntityData};
use geometry::{Quadrant, Vec3};
use matter_tree::MatterTree;
use space::Space;
use space_tree::SpaceTree;

use minifb::Key;
use std::cell::RefCell;
use std::rc::Rc;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

struct Colors {
    space_node: u32,
    matter_node: u32,
    player: u32,
    voxels: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

fn draw_matter_tree(
    colors: &Colors,
    buffer: &mut [u32],
    matter_area: &Rect,
    area: Rect,
    tree: &MatterTree,
) {
    for y in 0..area.h {
        for x in 0..area.w {
            let offset = (HEIGHT - 1 - (area.y + y)) * WIDTH + area.x + x;
            if x == 0 || y == 0 || x == area.w - 1 || y == area.h - 1 {
                buffer[offset] = colors.matter_node;
            }
        }
    }

    for entity in tree.entities.iter() {
        let pos = entity.bounding_sphere.center;
        let x = (pos.x as f64 + MatterTree::MAX_SIZE as f64 / 2.0f64) * matter_area.w as f64
            / MatterTree::MAX_SIZE as f64
            + matter_area.x as f64;
        let y = (pos.y as f64 + MatterTree::MAX_SIZE as f64 / 2.0f64) * matter_area.h as f64
            / MatterTree::MAX_SIZE as f64
            + matter_area.y as f64;
        let x = x as usize;
        let y = y as usize;
        let color = match entity.entity {
            EntityData::Player(_) => colors.player,
            EntityData::Voxels(_) => colors.voxels,
        };

        let dot_size: isize = usize::max(
            1,
            entity.bounding_sphere.radius as usize * matter_area.w / MatterTree::MAX_SIZE as usize,
        ) as isize;
        for y_i in
            isize::max(y as isize - dot_size, 0)..isize::min(y as isize + dot_size, HEIGHT as isize)
        {
            let y_shift = y_i - y as isize;
            let x_size = f32::sqrt((dot_size * dot_size - y_shift * y_shift) as f32) as isize;
            for x_i in
                isize::max(x as isize - x_size, 0)..isize::min(x as isize + x_size, WIDTH as isize)
            {
                let offset = (HEIGHT - 1 - y_i as usize) * WIDTH + x_i as usize;
                buffer[offset] = color;
            }
        }
    }

    for (i, sub_tree) in tree.sub_trees.iter().enumerate() {
        if let Some(sub_tree) = sub_tree {
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
            draw_matter_tree(colors, buffer, matter_area, sub_area, sub_tree);
        }
    }
}

fn draw_space_tree(colors: &Colors, buffer: &mut [u32], area: Rect, tree: &SpaceTree) {
    for y in 0..area.h {
        for x in 0..area.w {
            let offset = (HEIGHT - 1 - (area.y + y)) * WIDTH + area.x + x;
            if x == 0 || y == 0 || x == area.w - 1 || y == area.h - 1 {
                buffer[offset] = colors.space_node;
            }
        }
    }

    match tree {
        SpaceTree::Matter(matter) => draw_matter_tree(colors, buffer, &area, area, matter),
        SpaceTree::Parent(parent) => {
            for (i, sub_tree) in parent.sub_trees.iter().enumerate() {
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
                    draw_space_tree(colors, buffer, sub_area, tree)
                }
            }
        }
    }
}

fn draw_space(colors: &Colors, buffer: &mut [u32], space: &Space) {
    // Wipe board
    for i in buffer.iter_mut() {
        *i = 0x00000000;
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
        &space.tree.tree,
    );
}

fn main() {
    let mut space = Space::new();
    let player = Rc::new(RefCell::new(player::Player::new()));

    if let SpaceTree::Matter(matter) = space.tree.tree.as_mut() {
        matter.add_entities(vec![Box::new(Entity::new_player(
            Vec3 { x: 0, y: 0, z: 500 },
            player.clone(),
        ))]);
    }

    let colors = Colors {
        space_node: 0xFFFF0000,
        matter_node: 0xFF00FF00,
        voxels: 0xFF0080FF,
        player: 0xFF8000FF,
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

    // imit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    const DROP_BLOCK_COOLDOWN: usize = 60;
    let mut drop_block_cooldown = None;
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        {
            let mut control_dir = Vec3::ZERO;
            if window.is_key_down(Key::Right) {
                control_dir.x += 1;
            }
            if window.is_key_down(Key::Left) {
                control_dir.x -= 1;
            }
            if window.is_key_down(Key::Up) {
                control_dir.y += 1;
            }
            if window.is_key_down(Key::Down) {
                control_dir.y -= 1;
            }

            let mut player = player.borrow_mut();
            player.control(&control_dir);
            let replacement = match &mut drop_block_cooldown {
                None => {
                    if window.is_key_down(Key::Space) {
                        player.drop_block = true;
                        player.drop_block_fixed = window.is_key_down(Key::LeftCtrl);
                        Some(Some(DROP_BLOCK_COOLDOWN))
                    } else {
                        None
                    }
                }
                Some(n) => {
                    player.drop_block = false;
                    *n -= 1;
                    if *n == 0 {
                        Some(None)
                    } else {
                        None
                    }
                }
            };
            if let Some(replacement) = replacement {
                drop_block_cooldown = replacement;
            }
        }

        space.run();

        draw_space(&colors, &mut buffer, &space);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
