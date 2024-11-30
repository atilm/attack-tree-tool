use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use serde::Deserialize;
use thiserror::Error;

static OBJECT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn generate_id() -> u32 {
    OBJECT_COUNTER.fetch_add(1, Ordering::SeqCst) as u32
}

#[derive(Error, Debug, PartialEq)]
pub enum TreeError {
    #[error("Length mismatch between assessment vector and definition")]
    AssessmentVectorMismatch,
}

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

fn render(step: &dyn FeasibleStep, shape_str: &str) -> String {
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

pub struct AndNode {
    pub id: u32,
    pub description: String,
    pub parent: Option<Rc<dyn FeasibleStep>>,
    pub children: RefCell<Vec<Rc<dyn FeasibleStep>>>,
}

impl AndNode {
    pub fn new<F>(title: &str, parent: Option<Rc<dyn FeasibleStep>>, id_gen: F) -> AndNode
    where
        F: Fn() -> u32,
    {
        AndNode {
            id: id_gen(),
            description: title.to_string(),
            parent,
            children: RefCell::new(vec![]),
        }
    }
}

impl FeasibleStep for AndNode {
    fn id(&self) -> u32 {
        self.id
    }

    fn feasibility(&self) -> Result<FeasibilityAssessment, TreeError> {
        if self.children.borrow().is_empty() {
            return Err(TreeError::AssessmentVectorMismatch);
        }

        let maximum_assessment = self
            .children
            .borrow()
            .iter()
            .filter_map(|s| s.feasibility().ok())
            .reduce(|a, b| a.component_wise_max(&b).unwrap())
            .unwrap();

        Ok(maximum_assessment)
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
        render(self, " shape=trapezium")
    }

    fn get_children(&self) -> Vec<Rc<dyn FeasibleStep>> {
        let mut v = Vec::new();

        for c in self.children.borrow().iter() {
            v.push(c.clone())
        }

        v
    }
}

pub struct Leaf {
    pub id: u32,
    pub description: String,
    pub parent: Option<Rc<dyn FeasibleStep>>,
    pub criteria: FeasibilityAssessment,
}

impl Leaf {
    pub fn new<F>(
        description: &str,
        parent: Option<Rc<dyn FeasibleStep>>,
        definition: &Rc<FeasibilityCriteria>,
        assessment: &[u32],
        id_gen: F,
    ) -> Leaf
    where
        F: Fn() -> u32,
    {
        let assessments: Vec<Option<u32>> = assessment.iter().map(|v| Some(*v)).collect();

        Leaf {
            id: id_gen(),
            description: description.to_string(),
            parent,
            criteria: FeasibilityAssessment::new(definition, &assessments).unwrap(),
        }
    }
}

impl FeasibleStep for Leaf {
    fn id(&self) -> u32 {
        self.id
    }

    fn feasibility(&self) -> Result<FeasibilityAssessment, TreeError> {
        FeasibilityAssessment::new(&self.criteria.definition, &self.criteria.assessments.0)
    }

    fn title(&self) -> &str {
        &self.description
    }

    fn add_child(&self, _child: &Rc<dyn FeasibleStep>) {
        panic!("Attempt to add a child to an attack tree leaf.");
    }

    fn get_parent(&self) -> Option<Rc<dyn FeasibleStep>> {
        if let Some(s) = &self.parent {
            return Some(s.clone());
        }

        None
    }

    fn render(&self) -> String {
        render(self, "")
    }

    fn get_children(&self) -> Vec<Rc<dyn FeasibleStep>> {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct FeasibilityAssessment {
    definition: Rc<FeasibilityCriteria>,
    assessments: FeasibilityVector,
}

impl FeasibilityAssessment {
    pub fn new(
        definition: &Rc<FeasibilityCriteria>,
        assessments: &[Option<u32>],
    ) -> Result<FeasibilityAssessment, TreeError> {
        if assessments.len() != definition.0.len() {
            return Err(TreeError::AssessmentVectorMismatch);
        }

        Ok(FeasibilityAssessment {
            definition: Rc::clone(definition),
            assessments: FeasibilityVector(assessments.to_vec()),
        })
    }

    pub fn sum(&self) -> u32 {
        self.assessments.0.iter().map(|v| v.unwrap_or(0)).sum()
    }

    pub fn component_wise_max(
        &self,
        other: &FeasibilityAssessment,
    ) -> Result<FeasibilityAssessment, TreeError> {
        if self.assessments.0.len() != other.assessments.0.len() {
            return Err(TreeError::AssessmentVectorMismatch);
        }

        let maxima: Vec<Option<u32>> = self
            .assessments
            .0
            .iter()
            .zip(other.assessments.0.iter())
            .map(|(a, b)| Some(std::cmp::max(a.unwrap_or(0), b.unwrap_or(0))))
            .collect();

        FeasibilityAssessment::new(&self.definition, &maxima)
    }
}

#[derive(Clone, Debug)]
pub struct FeasibilityVector(Vec<Option<u32>>);

#[derive(Debug)]
pub struct FeasibilityCriteria(pub Vec<FeasiblityCriterion>);

#[derive(Deserialize, Debug)]
pub struct FeasiblityCriterion {
    pub name: String,
    pub id: String,
}

#[cfg(test)]
pub mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::model::TreeError;

    use super::{
        generate_id, AndNode, FeasibilityAssessment, FeasibilityCriteria, FeasibleStep,
        FeasiblityCriterion, Leaf, OrNode,
    };

    pub fn build_criteria(names: &[&str]) -> Rc<FeasibilityCriteria> {
        Rc::new(FeasibilityCriteria(
            names
                .iter()
                .map(|n| FeasiblityCriterion {
                    name: n.to_string(),
                    id: n.to_string(),
                })
                .collect(),
        ))
    }

    fn build_feasibility(
        definition: &Rc<FeasibilityCriteria>,
        assessments: &[u32],
    ) -> FeasibilityAssessment {
        let assessment_options: Vec<Option<u32>> = assessments.iter().map(|a| Some(*a)).collect();
        FeasibilityAssessment::new(definition, &assessment_options).unwrap()
    }

    fn build_leaf(criteria: &Rc<FeasibilityCriteria>, assessment: &[u32]) -> Leaf {
        let feasibility = build_feasibility(&criteria, assessment);

        Leaf {
            id: generate_id(),
            description: "Attack step".to_string(),
            parent: None,
            criteria: feasibility,
        }
    }

    fn build_and_node(children: Vec<Rc<dyn FeasibleStep>>) -> Rc<dyn FeasibleStep> {
        Rc::new(AndNode {
            id: generate_id(),
            description: "An and-node".to_string(),
            parent: None,
            children: RefCell::new(children),
        })
    }

    fn build_or_node(children: Vec<Rc<dyn FeasibleStep>>) -> Rc<dyn FeasibleStep> {
        Rc::new(OrNode {
            id: generate_id(),
            description: "An or-node".to_string(),
            parent: None,
            children: RefCell::new(children),
        })
    }

    #[test]
    fn in_feasibility_assessments_the_vector_must_match_the_definition() {
        let criteria = build_criteria(&["Eq", "Kn"]);

        let error_result =
            FeasibilityAssessment::new(&criteria, &[Some(1), Some(2), Some(3)]).unwrap_err();
        assert_eq!(error_result, TreeError::AssessmentVectorMismatch);
    }

    #[test]
    fn a_leaf_returns_its_feasibility_unmodified() {
        let criteria = build_criteria(&["Eq", "Kn"]);
        let leaf = build_leaf(&criteria, &[1, 2]);

        let result = leaf.feasibility().unwrap();

        let expected_feasibility = build_feasibility(&criteria, &[1, 2]);

        assert_eq!(result.assessments.0, expected_feasibility.assessments.0);
    }

    #[test]
    fn an_or_node_without_children_returns_an_error_for_feasibility() {
        let node = OrNode {
            id: generate_id(),
            description: "An or node".to_string(),
            parent: None,
            children: RefCell::new(vec![]),
        };

        assert_eq!(
            node.feasibility().unwrap_err(),
            TreeError::AssessmentVectorMismatch
        );
    }

    #[test]
    fn an_or_node_returns_the_minimum_feasibility_of_all_its_child_nodes() {
        let criteria = build_criteria(&["Eq", "Kn"]);

        let node = OrNode {
            id: generate_id(),
            description: "An or-node".to_string(),
            parent: None,
            children: RefCell::new(vec![
                Rc::new(build_leaf(&criteria, &[0, 50])),
                Rc::new(build_leaf(&criteria, &[1, 49])),
                Rc::new(build_leaf(&criteria, &[2, 3])),
            ]),
        };

        let expected_assessment = build_feasibility(&criteria, &[2, 3]);

        assert_eq!(
            node.feasibility().unwrap().assessments.0,
            expected_assessment.assessments.0
        );
    }

    #[test]
    fn an_or_node_returns_the_sum_of_its_feasibility_as_value() {
        let criteria = build_criteria(&["Eq", "Kn"]);

        let node = OrNode {
            id: generate_id(),
            description: "An or-node".to_string(),
            parent: None,
            children: RefCell::new(vec![
                Rc::new(build_leaf(&criteria, &[0, 50])),
                Rc::new(build_leaf(&criteria, &[1, 49])),
                Rc::new(build_leaf(&criteria, &[2, 3])),
            ]),
        };

        assert_eq!(node.feasibility_value(), 2 + 3);
    }

    #[test]
    fn an_and_node_without_children_returns_an_error_for_feasibility() {
        let node = AndNode {
            id: generate_id(),
            description: "An and-node".to_string(),
            parent: None,
            children: RefCell::new(vec![]),
        };

        assert_eq!(
            node.feasibility().unwrap_err(),
            TreeError::AssessmentVectorMismatch
        );
    }

    #[test]
    fn an_and_node_without_children_returns_0_as_feasibility_value() {
        let node = AndNode {
            id: generate_id(),
            description: "An and-node".to_string(),
            parent: None,
            children: RefCell::new(vec![]),
        };

        assert_eq!(node.feasibility_value(), 0);
    }

    #[test]
    fn an_and_node_returns_a_feasibility_with_maximum_components_of_all_children() {
        let criteria = build_criteria(&["Eq", "Kn", "WO"]);

        let node = AndNode {
            id: generate_id(),
            description: "An and-node".to_string(),
            parent: None,
            children: RefCell::new(vec![
                Rc::new(build_leaf(&criteria, &[1, 6, 8])),
                Rc::new(build_leaf(&criteria, &[2, 4, 9])),
                Rc::new(build_leaf(&criteria, &[3, 5, 7])),
            ]),
        };

        let expected_assessment = build_feasibility(&criteria, &[3, 6, 9]);

        assert_eq!(
            node.feasibility().unwrap().assessments.0,
            expected_assessment.assessments.0
        );
    }

    #[test]
    fn an_and_node_returns_the_sum_of_its_feasibility_as_value() {
        let criteria = build_criteria(&["Eq", "Kn", "WO"]);

        let node = AndNode {
            id: generate_id(),
            description: "An and-node".to_string(),
            parent: None,
            children: RefCell::new(vec![
                Rc::new(build_leaf(&criteria, &[1, 6, 8])),
                Rc::new(build_leaf(&criteria, &[2, 4, 9])),
                Rc::new(build_leaf(&criteria, &[3, 5, 7])),
            ]),
        };

        assert_eq!(node.feasibility_value(), 3 + 6 + 9);
    }

    #[test]
    fn a_leaf_returns_the_sum_of_all_assessments_as_feasibility_value() {
        let criteria = build_criteria(&["Eq", "Kn"]);
        let leaf = build_leaf(&criteria, &[1, 2]);

        let result = leaf.feasibility_value();

        assert_eq!(result, 3);
    }

    #[test]
    fn the_feasibility_of_a_three_level_tree_is_calculated_correctly() {
        let criteria = build_criteria(&["Eq", "Kn"]);

        // 3, 14
        let tree = build_and_node(vec![
            // 3, 5
            build_and_node(vec![
                Rc::new(build_leaf(&criteria, &[1, 5])),
                Rc::new(build_leaf(&criteria, &[3, 1])),
            ]),
            // 2, 14
            build_or_node(vec![
                Rc::new(build_leaf(&criteria, &[2, 14])),
                Rc::new(build_leaf(&criteria, &[20, 1])),
            ]),
        ]);

        let assessment = tree.feasibility().unwrap();

        let expected_assessment = build_feasibility(&criteria, &[3, 14]);

        assert_eq!(assessment.assessments.0, expected_assessment.assessments.0);

        assert_eq!(tree.feasibility_value(), 3 + 14);
    }
}
