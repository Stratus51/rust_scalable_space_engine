use crate::entity::Entity;
use crate::geometry::{Direction, FineDirection, Quadrant, Vec3, NB_DIRECTIONS, NB_QUADRANTS};
use crate::matter_tree::MatterTree;

#[derive(Debug, Clone, PartialEq)]
pub enum SpaceTree {
    Parent(SpaceTreeParent),
    Matter(MatterTree),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpaceTreeParent {
    pub scale: u32,
    pub sub_trees: [Option<Box<SpaceTree>>; NB_QUADRANTS],
}

impl SpaceTreeParent {
    fn build_sub_tree(&self) -> Box<SpaceTree> {
        Box::new(if self.scale == 0 {
            SpaceTree::Matter(MatterTree::new())
        } else {
            SpaceTree::Parent(SpaceTreeParent {
                scale: self.scale - 1,
                sub_trees: [SpaceTree::NONE_SPACE_CELL; NB_QUADRANTS],
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct EntityToDisplaceUp {
    path: Vec<Quadrant>,
    direction: Vec3,
    entity: Box<Entity>,
}

impl From<EntityToDisplaceUp> for EntityToDisplaceDown {
    fn from(up: EntityToDisplaceUp) -> Self {
        Self {
            path: up.path,
            entity: up.entity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct EntityToDisplaceDown {
    path: Vec<Quadrant>,
    entity: Box<Entity>,
}

impl SpaceTree {
    const NONE_SPACE_CELL: Option<Box<Self>> = None;

    pub fn new() -> Self {
        Self::Matter(MatterTree::new())
    }

    fn new_parent(&self) -> Self {
        let scale = match self {
            Self::Parent(child) => child.scale + 1,
            Self::Matter(_) => 0,
        };
        let sub_trees = [Self::NONE_SPACE_CELL; NB_QUADRANTS];
        Self::Parent(SpaceTreeParent { scale, sub_trees })
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Parent(parent) => parent.sub_trees.iter().all(|cell| cell.is_none()),
            Self::Matter(matter) => matter.is_empty(),
        }
    }

    fn get_displaced_outsider(mut entity: Box<Entity>) -> EntityToDisplaceUp {
        let direction = FineDirection::outsider_direction_vec(
            &entity.bounding_sphere.center,
            MatterTree::MAX_SIZE,
        );
        entity.switch_space_tree(direction, MatterTree::MAX_SIZE);
        EntityToDisplaceUp {
            path: vec![],
            direction,
            entity,
        }
    }

    fn relocate_entities(&mut self, entities: Vec<EntityToDisplaceDown>) {
        match self {
            Self::Matter(matter) => {
                matter.add_entities(entities.into_iter().map(|e| e.entity).collect());
            }
            Self::Parent(parent) => {
                let mut relocate = vec![vec![]; NB_QUADRANTS];
                for mut entity in entities.into_iter() {
                    let i = entity.path.pop().unwrap() as usize;
                    relocate[i].push(entity);
                }
                for (i, entities) in relocate.into_iter().enumerate() {
                    if !entities.is_empty() {
                        if parent.sub_trees[i].is_none() {
                            parent.sub_trees[i] = Some(parent.build_sub_tree());
                        }
                        // TODO Is there a cleaner Rust way to write this?
                        parent.sub_trees[i]
                            .as_mut()
                            .unwrap()
                            .relocate_entities(entities);
                    }
                }
            }
        }
    }

    fn run_actions(&mut self) {
        match self {
            Self::Matter(matter) => matter.run_actions(),
            Self::Parent(tree) => {
                for sub_tree in tree.sub_trees.iter_mut() {
                    if let Some(tree) = sub_tree {
                        tree.run_actions();
                    }
                }
            }
        }
    }

    fn run_movements(&mut self) {
        match self {
            Self::Matter(matter) => matter.run_movements(),
            Self::Parent(tree) => {
                for sub_tree in tree.sub_trees.iter_mut() {
                    if let Some(tree) = sub_tree {
                        tree.run_movements();
                    }
                }
            }
        }
    }

    fn apply_neighbourhood_collisions(&mut self) {
        match self {
            Self::Matter(matter) => matter.apply_neighbourhood_collisions(),
            Self::Parent(tree) => {
                for sub_tree in tree.sub_trees.iter_mut() {
                    if let Some(tree) = sub_tree {
                        tree.apply_neighbourhood_collisions();
                    }
                }
            }
        }
    }

    fn apply_external_collisions(&mut self, outsiders: &mut [&mut Box<Entity>]) {
        match self {
            Self::Matter(matter) => matter.apply_external_collisions(outsiders),
            Self::Parent(tree) => {
                for sub_tree in tree.sub_trees.iter_mut() {
                    if let Some(tree) = sub_tree {
                        tree.apply_external_collisions(outsiders);
                    }
                }
            }
        }
    }

    fn apply_inter_neighbourhood_collisions(
        &mut self,
    ) -> Vec<(&mut Box<Entity>, Vec<FineDirection>)> {
        match self {
            Self::Matter(matter) => matter.get_entities_touching_outside(),
            Self::Parent(parent) => {
                // TODO Not working. Requires full refactor.
                // let mut outsiders = vec![];
                // let mut insiders = vec![];
                // for (quad_i, sub_tree) in parent.sub_trees.iter_mut().enumerate() {
                //     let quad: Quadrant = num::FromPrimitive::from_usize(quad_i).unwrap();
                //     if let Some(tree) = sub_tree {
                //         for (overflower, dirs) in tree.apply_inter_neighbourhood_collisions() {
                //             let mut inside_quadrants = vec![];
                //             let mut remaining_dirs = vec![];
                //             for dir in dirs.into_iter() {
                //                 if let Some(dest_quad) = quad.move_to(dir.equivalent_vec()) {
                //                     inside_quadrants.push(dest_quad);
                //                 } else {
                //                     remaining_dirs.push(dir);
                //                 }
                //             }
                //             if inside_quadrants.is_empty() {
                //                 outsiders.push((overflower, remaining_dirs));
                //             } else {
                //                 insiders.push((overflower, inside_quadrants, remaining_dirs));
                //             }
                //         }
                //     }
                // }

                // // TODO See if there is a safe way to keep this optimization
                // unsafe {
                //     for i in 0..NB_QUADRANTS {
                //         let quad = num::FromPrimitive::from_usize(i).unwrap();
                //         let mut insiders: Vec<_> = insiders
                //             .iter_mut()
                //             .filter_map(|(entity, target_quads, _)| {
                //                 if target_quads.contains(&quad) {
                //                     Some(*entity)
                //                 } else {
                //                     None
                //                 }
                //             })
                //             .collect();
                //         if !insiders.is_empty() {
                //             let parent = parent as *mut SpaceTreeParent;
                //             if let Some(tree) = (*parent).sub_trees[i].as_mut() {
                //                 tree.apply_external_collisions(&mut insiders[..]);
                //             }
                //         }
                //     }
                // }

                // for (insider, _, dirs) in insiders.into_iter() {
                //     if !dirs.is_empty() {
                //         outsiders.push((insider, dirs));
                //     }
                // }

                // outsiders
                vec![]
            }
        }
    }

    fn refresh(&mut self) -> Vec<EntityToDisplaceUp> {
        match self {
            Self::Matter(cell) => {
                let outsiders = cell.refresh();
                outsiders
                    .into_iter()
                    .map(Self::get_displaced_outsider)
                    .collect()
            }
            Self::Parent(parent) => {
                let mut outsiders = vec![];
                let mut relocate = vec![vec![]; NB_QUADRANTS];
                for (i, child) in parent.sub_trees.iter_mut().enumerate() {
                    if let Some(child) = child {
                        let quadrant: Quadrant = num::FromPrimitive::from_usize(i).unwrap();
                        let sub_outsiders = child.refresh();
                        for mut displaced_outsider in sub_outsiders.into_iter() {
                            if let Some(relocation) = quadrant.move_to(displaced_outsider.direction)
                            {
                                relocate[relocation as usize].push(displaced_outsider.into());
                            } else {
                                // Add quadrant to dive in on the down path
                                let mirror_quadrant = quadrant.mirror(displaced_outsider.direction);
                                displaced_outsider.path.push(mirror_quadrant);

                                // Remove directions handled by that quadrant move
                                displaced_outsider.direction = displaced_outsider
                                    .direction
                                    .remove_matching_quadrant_component(mirror_quadrant);

                                outsiders.push(displaced_outsider);
                            }
                        }
                    }
                }
                for (i, entities) in relocate.into_iter().enumerate() {
                    if !entities.is_empty() {
                        if parent.sub_trees[i].is_none() {
                            parent.sub_trees[i] = Some(parent.build_sub_tree());
                        }
                        // TODO Is there a cleaner Rust way to write this?
                        let sub_tree = parent.sub_trees[i].as_mut().unwrap();
                        sub_tree.relocate_entities(entities);
                    }
                }
                outsiders
            }
        }
    }

    fn clean_empty_children(&mut self) {
        if let Self::Parent(parent) = self {
            // Clean empty quadrants
            for i in 0..NB_QUADRANTS {
                let mut need_emptying = false;
                if let Some(quad) = parent.sub_trees[i].as_mut() {
                    quad.clean_empty_children();
                    if quad.is_empty() {
                        need_emptying = true;
                    }
                }
                if need_emptying {
                    parent.sub_trees[i] = None;
                }
            }
        }
    }

    fn nb_nodes(&self) -> usize {
        match self {
            Self::Matter(_) => 1,
            Self::Parent(parent) => parent
                .sub_trees
                .iter()
                .map(|opt| match opt {
                    Some(tree) => tree.nb_nodes(),
                    None => 0,
                })
                .sum(),
        }
    }

    fn nb_matter_nodes(&self) -> usize {
        match self {
            Self::Matter(matter) => matter.nb_nodes(),
            Self::Parent(parent) => parent
                .sub_trees
                .iter()
                .map(|opt| match opt {
                    Some(tree) => tree.nb_nodes(),
                    None => 0,
                })
                .sum(),
        }
    }

    fn nb_entities(&self) -> usize {
        match self {
            Self::Matter(matter) => matter.nb_entities(),
            Self::Parent(parent) => parent
                .sub_trees
                .iter()
                .map(|opt| match opt {
                    Some(tree) => tree.nb_entities(),
                    None => 0,
                })
                .sum(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrowableSpaceTree {
    pub tree: Box<SpaceTree>,
}

impl GrowableSpaceTree {
    pub fn new() -> Self {
        Self {
            tree: Box::new(SpaceTree::new()),
        }
    }

    pub fn pick_expansion_quadrant(
        expansion_dirs: &mut [usize; NB_DIRECTIONS as usize],
    ) -> (Quadrant, usize) {
        let mut i_direction = 0;
        let mut dirs_consumed = 0;
        if expansion_dirs[Direction::Xp as usize] != 0 {
            expansion_dirs[Direction::Xp as usize] = 0;
            i_direction += 1 << 2;
            dirs_consumed += 1;
        } else if expansion_dirs[Direction::Xn as usize] != 0 {
            expansion_dirs[Direction::Xn as usize] = 0;
            dirs_consumed += 1;
        }

        if expansion_dirs[Direction::Yp as usize] != 0 {
            expansion_dirs[Direction::Yp as usize] = 0;
            i_direction += 1 << 1;
            dirs_consumed += 1;
        } else if expansion_dirs[Direction::Yn as usize] != 0 {
            expansion_dirs[Direction::Yn as usize] = 0;
            dirs_consumed += 1;
        }

        if expansion_dirs[Direction::Yp as usize] != 0 {
            expansion_dirs[Direction::Yp as usize] = 0;
            i_direction += 1 << 1;
            dirs_consumed += 1;
        } else if expansion_dirs[Direction::Yn as usize] != 0 {
            expansion_dirs[Direction::Yn as usize] = 0;
            dirs_consumed += 1;
        }
        let opposite_quadrant: Quadrant = num::FromPrimitive::from_usize(i_direction).unwrap();
        (opposite_quadrant.invert(), dirs_consumed)
    }

    pub fn run_actions(&mut self) {
        self.tree.run_actions();
    }

    pub fn run_movements(&mut self) {
        self.tree.run_movements();
    }

    pub fn refresh(&mut self) {
        let mut outsiders = self.tree.refresh();

        // Check in which directions the ousiders are
        let mut expansion_dirs = [0; NB_DIRECTIONS as usize];
        let mut nb_expansion_dirs = 0;
        for outsider in outsiders.iter() {
            let dirs = outsider.direction.direction_components();
            for dir in dirs.into_iter() {
                if expansion_dirs[dir as usize] == 0 {
                    nb_expansion_dirs += 1;
                }
                expansion_dirs[dir as usize] += 1;
            }
        }

        // While some outsiders are outside
        if outsiders.len() > nb_expansion_dirs {
            panic!("{} | {:?}", nb_expansion_dirs, outsiders);
        }
        while nb_expansion_dirs > 0 {
            // Pick a direction for space growth
            let (child_quadrant, dirs_consumed) =
                Self::pick_expansion_quadrant(&mut expansion_dirs);
            nb_expansion_dirs -= dirs_consumed;

            // Create new parent cell
            let parent = self.tree.new_parent();
            let child = std::mem::replace(&mut self.tree, Box::new(parent));
            if let SpaceTree::Parent(parent) = self.tree.as_mut() {
                parent.sub_trees[child_quadrant as usize] = Some(child);
            }

            // Update outsiders path
            for outsider in outsiders.iter_mut() {
                let mirror_quadrant = child_quadrant.mirror(outsider.direction);
                outsider.path.push(mirror_quadrant);
            }

            // Add outsiders back
            let mut new_insiders = vec![];
            let opposite_quadrant = child_quadrant.invert();
            for i in (0..outsiders.len()).rev() {
                if opposite_quadrant.match_direction(outsiders[i].direction) {
                    let outsider = outsiders.remove(i);
                    new_insiders.push(outsider.into());
                }
            }

            self.tree.relocate_entities(new_insiders);
        }

        // Cleanup useless children levels
        self.tree.clean_empty_children();

        // Cleanup useless parent levels
        loop {
            let child = match self.tree.as_mut() {
                SpaceTree::Matter(_) => break,
                SpaceTree::Parent(parent) => {
                    if parent
                        .sub_trees
                        .iter()
                        .map(|tree| tree.is_some() as usize)
                        .sum::<usize>()
                        > 1
                    {
                        break;
                    } else {
                        let mut child = None;
                        for tree in parent.sub_trees.iter_mut() {
                            if let Some(tree) = tree.take() {
                                child = Some(tree);
                                break;
                            }
                        }
                        match child {
                            Some(child) => child,
                            None => break,
                        }
                    }
                }
            };
            self.tree = child;
        }
    }

    pub fn nb_nodes(&self) -> usize {
        self.tree.nb_nodes()
    }

    pub fn nb_matter_nodes(&self) -> usize {
        self.tree.nb_matter_nodes()
    }

    pub fn nb_entities(&self) -> usize {
        self.tree.nb_entities()
    }
}
