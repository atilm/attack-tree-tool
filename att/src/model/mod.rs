use std::rc::Rc;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TreeError {
    #[error("Length mismatch between assessment vector and definition")]
    AssessmentVectorMismatch
}

pub trait FeasibleStep {
    fn feasibility(&self) -> Result<FeasibilityAssessment, TreeError>;
}

pub struct OrNode {
    pub description: String,
    pub children: Vec<Box<dyn FeasibleStep>>,
}

impl FeasibleStep for OrNode {
    fn feasibility(&self) -> Result<FeasibilityAssessment, TreeError> {
        if self.children.is_empty() {
            return Err(TreeError::AssessmentVectorMismatch)
        }
        
        let min_feasibility = self.children.iter()
            .map(|s| s.feasibility().unwrap())
            .min_by_key(|f| f.sum());

        Ok(min_feasibility.unwrap())
    }
}

pub struct Leaf {
    pub description: String,
    pub criteria: FeasibilityAssessment,
}

impl FeasibleStep for Leaf {
    fn feasibility(&self) -> Result<FeasibilityAssessment, TreeError> {
        FeasibilityAssessment::new(
            &self.criteria.definition,
            &self.criteria.assessments.0)
    }
}

#[derive(Debug)]
pub struct FeasibilityAssessment {
    definition: Rc<FeasibilityCriteria>,
    assessments: FeasibilityVector,
}

impl FeasibilityAssessment {
    pub fn new(definition: &Rc<FeasibilityCriteria>, assessments: &[u32]) -> Result<FeasibilityAssessment, TreeError> {
        if assessments.len() != definition.0.len() {
            return Err(TreeError::AssessmentVectorMismatch)
        }

        Ok(FeasibilityAssessment {
            definition: Rc::clone(definition),
            assessments: FeasibilityVector(assessments.to_vec())
        })
    }

    pub fn sum(&self) -> u32 {
        self.assessments.0.iter().sum()
    }
}

#[derive(Clone, Debug)]
pub struct FeasibilityVector(Vec<u32>);

#[derive(Debug)]
pub struct FeasibilityCriteria(Vec<FeasiblityCriterion>);

#[derive(Debug)]
pub struct FeasiblityCriterion {
    name: String,
    id: String,
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::model::TreeError;

    use super::{
        FeasibilityAssessment, FeasibilityCriteria, FeasibilityVector, FeasibleStep,
        FeasiblityCriterion, Leaf, OrNode,
    };

    fn build_criteria(names: &[&str]) -> Rc<FeasibilityCriteria> {
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
        FeasibilityAssessment::new(definition, assessments).unwrap()
    }

    fn build_leaf(criteria: &Rc<FeasibilityCriteria>, assessment: &[u32]) -> Leaf {
        let feasibility = build_feasibility(&criteria, &assessment);

        Leaf {
            description: "Attack step".to_string(),
            criteria: feasibility,
        }
    }

    #[test]
    fn in_feasibility_assessments_the_vector_must_match_the_definition() {
        let criteria = build_criteria(&["Eq", "Kn"]);

        let error_result = 
            FeasibilityAssessment::new(&criteria, &[1, 2, 3]).unwrap_err();
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
            children: vec![]
        };

        assert_eq!(node.feasibility().unwrap_err(), TreeError::AssessmentVectorMismatch);
    }

    #[test]
    fn an_or_node_returns_the_minimum_feasibility_of_all_its_child_nodes() {
        let criteria = build_criteria(&["Eq", "Kn"]);

        let node = OrNode {
            description: "An or node".to_string(),
            children: vec![
                Box::new(build_leaf(&criteria, &[0, 50])),
                Box::new(build_leaf(&criteria, &[1, 49])),
                Box::new(build_leaf(&criteria, &[2, 3]))
            ]
        };

        let expected_assessment = build_feasibility(&criteria, &[2, 3]);

        assert_eq!(node.feasibility().unwrap().assessments.0, expected_assessment.assessments.0);
    }
}
