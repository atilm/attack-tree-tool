use std::rc::Rc;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TreeError {
    #[error("Length mismatch between assessment vector and definition")]
    AssessmentVectorMismatch
}

pub trait FeasibleStep {
    fn feasibility(&self) -> FeasibilityAssessment;
}

pub struct OrNode {
    pub description: String,
    pub children: Vec<Box<dyn FeasibleStep>>,
}

pub struct Leaf {
    pub description: String,
    pub criteria: FeasibilityAssessment,
}

impl FeasibleStep for Leaf {
    fn feasibility(&self) -> FeasibilityAssessment {
        FeasibilityAssessment::new(
            &self.criteria.definition,
            &self.criteria.assessments.0).unwrap()
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
        FeasiblityCriterion, Leaf,
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
        let feasibility = build_feasibility(&criteria, &[1, 2]);

        let leaf = Leaf {
            description: "Attack step".to_string(),
            criteria: feasibility,
        };

        let result = leaf.feasibility();

        let expected_feasibility = build_feasibility(&criteria, &[1, 2]);

        assert_eq!(result.assessments.0, expected_feasibility.assessments.0);
    }
}
