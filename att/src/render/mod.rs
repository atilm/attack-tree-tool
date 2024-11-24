use std::rc::Rc;
use thiserror::Error;

use crate::model::{FeasibleStep, TreeError};

#[derive(Error, Debug, PartialEq)]
pub enum RenderError {
    #[error("File write error")]
    FileWriteError,
}

pub fn render_to_file(root_node: &Rc<dyn FeasibleStep>, file_path: &str) -> Result<(), RenderError> {
    let dot_file_content = render_to_dot_file(root_node)?;

    

    Ok(())
}

pub fn render_to_dot_file(root_node: &Rc<dyn FeasibleStep>) -> Result<String, RenderError> {
    Ok("".to_string())
}
