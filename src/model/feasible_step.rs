use std::rc::Rc;

use super::{FeasibilityAssessment, TreeError};

pub trait FeasibleStep {
    fn id(&self) -> u32;
    
    // todo: add_child does not make sense for leafs. What would be a better design?
    fn add_child(&self, child: &Rc<dyn FeasibleStep>);

    fn get_parent(&self) -> Option<Rc<dyn FeasibleStep>>;

    fn title(&self) -> &str;

    fn feasibility_value(&self) -> u32 {
        let feasibility = self.feasibility();
        match feasibility {
            Ok(f) => f.sum(),
            Err(_) => 0,
        }
    }

    fn feasibility(&self) -> Result<FeasibilityAssessment, TreeError>;

    fn render(&self) -> String;

    fn get_children(&self) -> Vec<Rc<dyn FeasibleStep>>;
}