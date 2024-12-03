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

pub fn render(step: &dyn FeasibleStep, shape_str: &str) -> String {
    let assessment = step.feasibility();

    if assessment.is_err() {
        return format!(r#"label="{}"#, step.title());
    }

    let assessment = assessment.unwrap();
    let assessment_strings: Vec<String> = assessment
        .definition
        .0
        .iter()
        .zip(assessment.assessments.0)
        .map(|(c, v)| format!("{}={}", c.id, v.unwrap_or(0)))
        .collect();

    format!(
        r#"label="{}\n{}\n{}"{}"#,
        step.title(),
        step.feasibility_value(),
        assessment_strings.join(", "),
        shape_str
    )
}
