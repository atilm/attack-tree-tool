use std::{collections::HashMap, io::BufRead, rc::Rc};

use crate::model::*;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TreeFileError {
    #[error("File read error")]
    FileReadError,
    #[error("Syntax error")]
    SyntaxError(u32),
}

pub enum ParserState {
    InTitle,
    InAssessmentName,
    InAssessmentValue
}

struct AttackTreeParser {
    state: ParserState,
    title: String,
    assessment_value: String,
    assessment_title: String,
    assessments: HashMap<String, u32>
}

impl AttackTreeParser {
    pub fn new() -> AttackTreeParser {
        AttackTreeParser {
            state: ParserState::InTitle,
            title: String::new(),
            assessment_value: String::new(),
            assessment_title: String::new(),
            assessments: HashMap::new()
        }
    }

    pub fn parse(&mut self, buf_read: &mut dyn BufRead, definition: &Rc<FeasibilityCriteria>) -> Result<Box<dyn FeasibleStep>, TreeFileError> {
        let mut text = String::new();
        if buf_read.read_to_string(&mut text).is_err() {
            return Err(TreeFileError::FileReadError);
        }
        
        for c in text.chars() {
            match self.state {
                ParserState::InTitle => {
                    if c == ';' {
                        self.state = ParserState::InAssessmentName;
                        self.assessment_value.clear();
                        self.assessment_title.clear();
                    }
                    else {
                        self.title.push(c);
                    }
                },
                ParserState::InAssessmentName => {
                    if c == '=' {
                        self.state = ParserState::InAssessmentValue;
                    }
                    else {
                        self.assessment_title.push(c);
                    }
                },
                ParserState::InAssessmentValue => {
                    if c == ',' {
                        self.commit_assessment()?;
                        self.state = ParserState::InAssessmentName;
                    }
                    else {
                        self.assessment_value.push(c);
                    }
                }
            }
        }
    
        // handle leafs at end of file
        if let ParserState::InAssessmentValue = self.state {
            self.commit_assessment()?;
        }
    
        let assessment_values: Vec<u32> = definition.0.iter()
            .map(|c| &c.name)
            .filter_map(|n| self.assessments.get(n))
            .map(|v| *v)
            .collect();
        
        Ok(Box::new(Leaf {
            description: self.title.clone(),
            criteria: FeasibilityAssessment::new(&definition, &assessment_values).unwrap()
        }))
    }

    fn commit_assessment(&mut self) -> Result<(), TreeFileError> {
        let value: u32 = match self.assessment_value.parse() {
            Ok(v) => v,
            Err(_) => { return Err(TreeFileError::SyntaxError(1)); }
        };

        self.assessments.insert(self.assessment_title.trim().to_string(), value);
        self.assessment_value.clear();
        self.assessment_title.clear();

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::io;
    use crate::model::tests::*;
    use super::*;

// file read error
// missing semicolon
// Wrong category
// no =

#[test]
fn read_a_file_with_one_leaf() {
    let definition = build_criteria(&["Eq", "Kn"]);

    let mut file_stub = io::Cursor::new(r#"Break into house;  Kn=5, Eq=3"#);

    let mut parser = AttackTreeParser::new();

    let result = parser.parse(&mut file_stub, &definition).unwrap();

    assert_eq!(result.feasibility_value(), 3 + 5);
    assert_eq!(result.title(), "Break into house")
}

#[test]
fn errors_in_assessment_value_formats_are_handled() {
    let definition = build_criteria(&["Eq", "Kn"]);

    // assessments should be integers
    let mut file_stub = io::Cursor::new(r#"Break into house;  Kn=5.1, Eq=3"#);

    let mut parser = AttackTreeParser::new();

    let result = parser.parse(&mut file_stub, &definition);

    assert_eq!(result.err(), Some(TreeFileError::SyntaxError(1)))
}

}