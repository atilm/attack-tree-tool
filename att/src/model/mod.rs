use std::rc::Rc;


pub trait FeasibleStep {
    fn feasibility(&self) -> FeasibilityAssessment;
}

pub struct OrNode {
    pub description: String,
    pub children: Vec<Box<dyn FeasibleStep>>
}

pub struct Leaf {
    pub description: String,
    pub criteria: FeasibilityAssessment
}

impl FeasibleStep for Leaf {
    fn feasibility(&self) -> FeasibilityAssessment {
        FeasibilityAssessment {
            definition: Rc::clone(&self.criteria.definition),
            assessments: self.criteria.assessments.clone()
        }
    }
}

pub struct FeasibilityAssessment {
    definition: Rc<FeasibilityCriteria>,
    assessments: FeasibilityVector
}

#[derive(Clone)]
pub struct FeasibilityVector(Vec<u32>);

pub struct FeasibilityCriteria(Vec<FeasiblityCriterion>);

pub struct FeasiblityCriterion {
    name: String,
    id: String
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{FeasibilityAssessment, FeasibilityCriteria, FeasibilityVector, FeasiblityCriterion, Leaf, FeasibleStep};

    fn build_criteria(names: &[&str]) -> Rc<FeasibilityCriteria> {
        Rc::new(FeasibilityCriteria(
            names.iter().map(|n| FeasiblityCriterion { name: n.to_string(), id: n.to_string()}).collect()
        ))
    }

    fn build_feasibility(criteria: &Rc<FeasibilityCriteria>, assessments: &[u32]) -> FeasibilityAssessment {
        FeasibilityAssessment {
            definition: Rc::clone(&criteria),
            assessments: FeasibilityVector(assessments.to_vec())
        }
    }

    #[test]
    fn a_leaf_returns_its_feasibility_unmodified() {
        let criteria = build_criteria(&["Eq", "Kn"]);
        let feasibility = build_feasibility(&criteria, &[1, 2]);

        let leaf = Leaf {
            description: "Attack step".to_string(),
            criteria: feasibility
        };

        let result = leaf.feasibility();

        let expected_feasibility = build_feasibility(&criteria, &[1, 2]);

        assert_eq!(result.assessments.0, expected_feasibility.assessments.0);
    }
}
