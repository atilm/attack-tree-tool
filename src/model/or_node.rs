use std::{cell::RefCell, rc::Rc};

use super::{render, FeasibilityAssessment, FeasibleStep, TreeError};


pub struct OrNode {
    pub id: u32,
    pub description: String,
    pub parent: Option<Rc<dyn FeasibleStep>>,
    pub children: RefCell<Vec<Rc<dyn FeasibleStep>>>,
}

impl OrNode {
    pub fn new<F>(title: &str, parent: Option<Rc<dyn FeasibleStep>>, id_gen: F) -> OrNode
    where
        F: Fn() -> u32,
    {
        OrNode {
            id: id_gen(),
            description: title.to_string(),
            parent,
            children: RefCell::new(vec![]),
        }
    }
}

impl FeasibleStep for OrNode {
    fn id(&self) -> u32 {
        self.id
    }

    fn feasibility(&self) -> Result<FeasibilityAssessment, TreeError> {
        if self.children.borrow().is_empty() {
            return Err(TreeError::AssessmentVectorMismatch);
        }

        let min_feasibility = self
            .children
            .borrow()
            .iter()
            .map(|s| s.feasibility().unwrap())
            .min_by_key(|f| f.sum());

        Ok(min_feasibility.unwrap())
    }

    fn title(&self) -> &str {
        &self.description
    }

    fn add_child(&self, child: &Rc<dyn FeasibleStep>) {
        self.children.borrow_mut().push(child.clone());
    }

    fn get_parent(&self) -> Option<Rc<dyn FeasibleStep>> {
        if let Some(s) = &self.parent {
            return Some(s.clone());
        }

        None
    }

    fn render(&self) -> String {
        render(self, " shape=invtrapezium")
    }

    fn get_children(&self) -> Vec<Rc<dyn FeasibleStep>> {
        let mut v = Vec::new();

        for c in self.children.borrow().iter() {
            v.push(c.clone())
        }

        v
    }
}
