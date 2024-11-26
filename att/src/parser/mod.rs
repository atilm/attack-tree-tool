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

enum ParserState {
    DeterminingIndentationLevel,
    InTitle,
    DeterminingNodeType,
    InAssessmentName,
    InAssessmentValue,
    SkipToLineEnd,
}

enum NodeType {
    Unknown,
    AndNode,
    OrNode,
    Leaf,
}

pub struct AttackTreeParser {
    state: ParserState,
    title: String,
    assessment_value: String,
    assessment_title: String,
    parsed_assessments: HashMap<String, u32>,
    current_node_type: NodeType,
    indentation_counter: u32,
    previous_indentation: u32,
    current_indentation: u32,
    current_node: Option<Rc<dyn FeasibleStep>>,
    last_added_node: Option<Rc<dyn FeasibleStep>>,
}

impl AttackTreeParser {
    pub fn new() -> AttackTreeParser {
        AttackTreeParser {
            state: ParserState::DeterminingIndentationLevel,
            title: String::new(),
            assessment_value: String::new(),
            assessment_title: String::new(),
            parsed_assessments: HashMap::new(),
            current_node_type: NodeType::Unknown,
            indentation_counter: 0,
            previous_indentation: 0,
            current_indentation: 0,
            current_node: None,
            last_added_node: None,
        }
    }

    pub fn parse(
        &mut self,
        buf_read: &mut dyn BufRead,
        definition: &Rc<FeasibilityCriteria>,
    ) -> Result<Rc<dyn FeasibleStep>, TreeFileError> {
        let mut text = String::new();
        if buf_read.read_to_string(&mut text).is_err() {
            return Err(TreeFileError::FileReadError);
        }

        for c in text.chars() {
            match self.state {
                ParserState::InTitle => {
                    if c == ';' {
                        self.set_state(ParserState::DeterminingNodeType);
                    } else {
                        self.title.push(c);
                    }
                }
                ParserState::DeterminingNodeType => {
                    if c == '&' {
                        self.current_node_type = NodeType::AndNode;
                        self.add_node(Rc::new(AndNode::new(
                            &self.title,
                            self.current_node.clone(),
                            generate_id,
                        )));
                        self.state = ParserState::SkipToLineEnd;
                        self.set_state(ParserState::SkipToLineEnd);
                    } else if c == '|' {
                        self.current_node_type = NodeType::OrNode;
                        self.add_node(Rc::new(OrNode::new(
                            &self.title,
                            self.current_node.clone(),
                            generate_id,
                        )));
                        self.set_state(ParserState::SkipToLineEnd);
                    } else if c != ' ' {
                        self.current_node_type = NodeType::Leaf;
                        self.set_state(ParserState::InAssessmentName);
                        self.assessment_title.push(c);
                    }
                }
                ParserState::SkipToLineEnd => {
                    if c == '\n' {
                        self.set_state(ParserState::DeterminingIndentationLevel);
                    }
                }
                ParserState::DeterminingIndentationLevel => {
                    if c == ' ' {
                        self.indentation_counter += 1;
                    } else if c == '\n' {
                        self.set_state(ParserState::DeterminingIndentationLevel);
                    } else {
                        self.previous_indentation = self.current_indentation;
                        self.current_indentation = self.indentation_counter;

                        self.set_state(ParserState::InTitle);
                        self.title.push(c);
                    }
                }
                ParserState::InAssessmentName => {
                    if c == '=' {
                        self.set_state(ParserState::InAssessmentValue);
                    } else {
                        self.assessment_title.push(c);
                    }
                }
                ParserState::InAssessmentValue => {
                    if c == ',' {
                        self.commit_assessment()?;
                        self.set_state(ParserState::InAssessmentName);
                    } else if c == '\n' {
                        self.commit_assessment()?;
                        self.add_node(self.build_leaf(&definition));
                        self.set_state(ParserState::DeterminingIndentationLevel);
                    } else {
                        self.assessment_value.push(c);
                    }
                }
            }
        }

        // handle leafs at end of file
        if let ParserState::InAssessmentValue = self.state {
            self.commit_assessment()?;
            self.add_node(self.build_leaf(&definition));
        }

        // set self.current_node to the tree's root node
        // ToDo: just safe the root node in an extra variable
        loop {
            if let Some(n) = &self.current_node {
                if let Some(parent) = n.get_parent() {
                    self.current_node.replace(parent.clone());
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(self.current_node.as_ref().unwrap().clone())
    }

    fn set_state(&mut self, state: ParserState) {
        self.state = state;

        match self.state {
            ParserState::DeterminingIndentationLevel => {
                self.indentation_counter = 0;
            }
            ParserState::InTitle => {
                self.title.clear();
            }
            ParserState::DeterminingNodeType => {}
            ParserState::InAssessmentName => {
                self.assessment_title.clear();
            }
            ParserState::InAssessmentValue => {
                self.assessment_value.clear();
            }
            ParserState::SkipToLineEnd => {}
        }
    }

    fn add_node(&mut self, node: Rc<dyn FeasibleStep>) {
        if self.current_node.is_none() {
            self.current_node = Some(node.clone());
            self.last_added_node = Some(node.clone());
        } else {
            if self.current_indentation > self.previous_indentation {
                self.current_node
                    .replace(self.last_added_node.as_ref().unwrap().clone());
            }
            if self.current_indentation < self.previous_indentation {
                self.current_node
                    .replace(self.current_node.as_ref().unwrap().get_parent().unwrap());
            }

            self.current_node.as_ref().unwrap().add_child(&node);
            self.last_added_node.replace(node.clone());
        }
    }

    fn build_leaf(&self, definition: &Rc<FeasibilityCriteria>) -> Rc<dyn FeasibleStep> {
        let assessment_values: Vec<Option<u32>> = definition
            .0
            .iter()
            .map(|c| &c.name)
            .map(|n| self.parsed_assessments.get(n))
            .map(|v| match v {
                Some(v) => Some(*v),
                None => None,
            })
            .collect();

        Rc::new(Leaf {
            id: generate_id(),
            description: self.title.clone(),
            parent: self.current_node.clone(),
            criteria: FeasibilityAssessment::new(&definition, &assessment_values).unwrap(),
        })
    }

    fn commit_assessment(&mut self) -> Result<(), TreeFileError> {
        let value: u32 = match self.assessment_value.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(TreeFileError::SyntaxError(1));
            }
        };

        self.parsed_assessments
            .insert(self.assessment_title.trim().to_string(), value);
        self.assessment_value.clear();
        self.assessment_title.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tests::*;
    use std::io;

    // Unknown category

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

    #[test]
    fn an_and_node_with_two_leafs_can_be_parsed() {
        let definition = build_criteria(&["Eq", "Kn"]);

        let mut file_stub = io::Cursor::new(
            r#"
Break into house;&
    Observe when people are away; Kn=6, Eq=1
    Pick lock; Kn=5, Eq=3"#,
        );

        let mut parser = AttackTreeParser::new();

        let result = parser.parse(&mut file_stub, &definition).unwrap();

        assert_eq!(result.title(), "Break into house");
        assert_eq!(result.feasibility_value(), 6 + 3);
    }

    #[test]
    fn an_or_node_with_two_leafs_can_be_parsed() {
        let definition = build_criteria(&["Eq", "Kn"]);

        let mut file_stub = io::Cursor::new(
            r#"
Enter house;|
    Trick people; Kn=6, Eq=0
    Pick lock; Kn=5, Eq=3"#,
        );

        let mut parser = AttackTreeParser::new();

        let result = parser.parse(&mut file_stub, &definition).unwrap();

        assert_eq!(result.title(), "Enter house");
        assert_eq!(result.feasibility_value(), 6 + 0);
    }

    #[test]
    fn a_multi_level_tree_can_be_parsed() {
        let definition = build_criteria(&["Eq", "Kn"]);

        let mut file_stub = io::Cursor::new(
            r#"
Enter house;&
    Observe when people are away;|
        Step 1; Kn=15, Eq=5
        Step 2; Kn=1, Eq=3
    Break into the house;&
        Step 3; Kn=0, Eq=2
        Step 4; Kn=4, Eq=0"#,
        );

        let mut parser = AttackTreeParser::new();

        let result = parser.parse(&mut file_stub, &definition).unwrap();

        assert_eq!(result.title(), "Enter house");
        let children = result.get_children();
        for c in children {
            assert_eq!(c.get_parent().unwrap().id(), result.id());
        }

        assert_eq!(result.feasibility_value(), 4 + 3);
    }
}
