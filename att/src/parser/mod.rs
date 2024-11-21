use std::{collections::HashMap, io::BufRead, rc::Rc};

use crate::model::*;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TreeFileError {
    #[error("File read error")]
    FileReadError(u32),
    #[error("Syntax error")]
    SyntaxError(u32),
}

pub enum ParserState {
    InTitle,
    InAssessmentName,
    InAssessmentValue
}

fn parse_tree_file(buf_read: &mut dyn BufRead, definition: &Rc<FeasibilityCriteria>) -> Result<Box<dyn FeasibleStep>, TreeFileError> {
    let mut text = String::new();
    if buf_read.read_to_string(&mut text).is_err() {
        return Err(TreeFileError::FileReadError(1));
    }

    let mut state = ParserState::InTitle;
    let mut title = String::new();
    let mut current_assessment = String::new();
    let mut current_assessment_title = String::new();
    let mut assessments: HashMap<String, u32> = HashMap::new();
    
    for c in text.chars() {
        match state {
            ParserState::InTitle => {
                if c == ';' {
                    state = ParserState::InAssessmentName;
                    current_assessment.clear();
                    current_assessment_title.clear();
                }
                else {
                    title.push(c);
                }
            },
            ParserState::InAssessmentName => {
                if c == '=' {
                    state = ParserState::InAssessmentValue;
                }
                else {
                    current_assessment_title.push(c);
                }
            },
            ParserState::InAssessmentValue => {
                if c == ',' {
                    let value: u32 = current_assessment.parse().unwrap();
                    assessments.insert(current_assessment_title.trim().to_string(), value);
                    state = ParserState::InAssessmentName;
                    current_assessment.clear();
                    current_assessment_title.clear();
                }
                else {
                    current_assessment.push(c);
                }
            }
        }
    }

    let assessment_values: Vec<u32> = definition.0.iter()
        .map(|c| &c.name)
        .filter_map(|n| assessments.get(n))
        .map(|v| *v)
        .collect();
    
    Ok(Box::new(Leaf {
        description: title,
        criteria: FeasibilityAssessment::new(&definition, &assessment_values).unwrap()
    }))
}

#[cfg(test)]
mod tests {
    use std::io;
    use crate::model::tests::*;
    use super::*;

// file read error
// missing semicolon
// Wrong category
// not a number as value
// no =
// different order of Kn and Eq

#[test]
fn read_a_file_with_one_leaf() {
    let definition = build_criteria(&["Eq", "Kn"]);

    let mut file_stub = io::Cursor::new(r#"Break into house;  Kn=5, Eq=3,"#);

    let result = parse_tree_file(&mut file_stub, &definition).unwrap();

    assert_eq!(result.feasibility_value(), 3 + 5);
    assert_eq!(result.title(), "Break into house")
}

}