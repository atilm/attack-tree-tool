use std::{borrow::BorrowMut, cell::{Cell, RefCell}, ops::DerefMut, rc::Rc};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TreeError {
    #[error("Length mismatch between assessment vector and definition")]
    AssessmentVectorMismatch,
}


pub trait FeasibleStep {
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
}

pub struct OrNode {
    pub description: String,
    pub parent: Option<Rc<dyn FeasibleStep>>,
    pub children: RefCell<Vec<Rc<dyn FeasibleStep>>>,
}

impl OrNode {
    pub fn new(title: &str, parent: Option<Rc<dyn FeasibleStep>>) -> OrNode {
        OrNode {
            description: title.to_string(),
            parent,
            children: RefCell::new(vec![])
        }
    }
}

impl FeasibleStep for OrNode {
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
            return Some(s.clone())
        }

        None
    }
}

pub struct AndNode {
    pub description: String,
    pub parent: Option<Rc<dyn FeasibleStep>>,
    pub children: RefCell<Vec<Rc<dyn FeasibleStep>>>,
}

impl AndNode {
    pub fn new(title: &str, parent: Option<Rc<dyn FeasibleStep>>) -> AndNode {
        AndNode {
            description: title.to_string(),
            parent,
            children: RefCell::new(vec![])
        }
    }
}

impl FeasibleStep for AndNode {
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
            return Some(s.clone())
        }

        None
    }
}

pub struct Leaf {
    pub description: String,
    pub parent: Option<Rc<dyn FeasibleStep>>,
    pub criteria: FeasibilityAssessment,
}

impl FeasibleStep for Leaf {
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
            return Some(s.clone())
        }

        None
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

#[derive(Debug)]
pub struct FeasiblityCriterion {
    pub name: String,
    _id: String,
}

#[cfg(test)]
pub mod tests {
    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::model::TreeError;

    use super::{
        AndNode, FeasibilityAssessment, FeasibilityCriteria, FeasibleStep, FeasiblityCriterion,
        Leaf, OrNode,
    };

    pub fn build_criteria(names: &[&str]) -> Rc<FeasibilityCriteria> {
        Rc::new(FeasibilityCriteria(
            names
                .iter()
                .map(|n| FeasiblityCriterion {
                    name: n.to_string(),
                    _id: n.to_string(),
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
            description: "Attack step".to_string(),
            parent: None,
            criteria: feasibility,
        }
    }

    fn build_and_node(children: Vec<Rc<dyn FeasibleStep>>) -> Rc<dyn FeasibleStep> {
        Rc::new(AndNode {
            description: "An and-node".to_string(),
            parent: None,
            children: RefCell::new(children)
        })
    }

    fn build_or_node(children: Vec<Rc<dyn FeasibleStep>>) -> Rc<dyn FeasibleStep> {
        Rc::new(OrNode {
            description: "An or-node".to_string(),
            parent: None,
            children: RefCell::new(children)
        })
    }

    #[test]
    fn in_feasibility_assessments_the_vector_must_match_the_definition() {
        let criteria = build_criteria(&["Eq", "Kn"]);

        let error_result = FeasibilityAssessment::new(&criteria, &[Some(1), Some(2), Some(3)]).unwrap_err();
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
            description: "An or-node".to_string(),
            parent: None,
            children: RefCell::new(vec![
                Rc::new(build_leaf(&criteria, &[0, 50])),
                Rc::new(build_leaf(&criteria, &[1, 49])),
                Rc::new(build_leaf(&criteria, &[2, 3])),
            ]),
        };

        assert_eq!(
            node.feasibility_value(),
            2 + 3
        );
    }

    #[test]
    fn an_and_node_without_children_returns_an_error_for_feasibility() {
        let node = AndNode {
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
            ])
        ]);

        let assessment = tree.feasibility().unwrap();

        let expected_assessment = build_feasibility(&criteria, &[3, 14]);

        assert_eq!(assessment.assessments.0, expected_assessment.assessments.0);

        assert_eq!(tree.feasibility_value(), 3 + 14);
    }
}
