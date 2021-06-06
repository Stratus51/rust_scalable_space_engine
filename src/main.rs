extern crate num;
#[macro_use]
extern crate num_derive;

mod geometry;
use geometry::{Mat3, Vec3, NB_QUADRANTS};
mod space;
use space::{
    PonctualMass, Space, SpaceConfiguration, SpaceEntity, SpaceEntityCaracteristics,
    SpaceEntityData,
};

fn main() {
    let mut space = Space::new(SpaceConfiguration {
        tick_size: 10_000, // duration of a world tick in us

        // Optimization related
        gravity_threshold: 0, // cm/s^2
    });

    let mass_center = PonctualMass {
        pos: Vec3 {
            x: 50,
            y: 50,
            z: 50,
        },
        mass: 100,
    };
    space.insert_entity(SpaceEntity::new(SpaceEntityData {
        id: 1,

        // Localisation
        pos: Vec3::zero(),
        orientation: Mat3::identity(),

        // Dynamics
        speed: Vec3 { x: 100, y: 0, z: 0 },

        // Caracteristics
        caracteristics: SpaceEntityCaracteristics {
            size: Vec3 {
                x: 100,
                y: 100,
                z: 100,
            },
            full_mass: mass_center,
            split_mass: [mass_center; NB_QUADRANTS],
        },
    }));

    space.run();
    space.run();
    space.run();
    space.run();
}
