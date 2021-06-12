use crate::space_tree::GrowableSpaceTree;

#[derive(Debug, Clone, PartialEq)]
pub struct Space {
    pub tree: GrowableSpaceTree,
}

impl Space {
    pub fn new() -> Self {
        Self {
            tree: GrowableSpaceTree::new(),
        }
    }

    pub fn run(&mut self) {
        self.tree.run_movements();
        self.tree.refresh();
    }
}
