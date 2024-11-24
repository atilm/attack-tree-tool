use std::rc::Rc;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use thiserror::Error;

use crate::model::FeasibleStep;

#[derive(Error, Debug, PartialEq)]
pub enum RenderError {
    #[error("File write error")]
    FileWriteError,
}

pub fn render_to_png(root_node: &Rc<dyn FeasibleStep>, file_path: &str) -> std::io::Result<()> {
    let dot_file_content = render_to_dot_string(root_node).expect("render to dot-file error");

    let mut child = Command::new("dot")
        .args(["-Tpng", "-o", file_path])
        .stdin(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(dot_file_content.as_bytes())?;

    Ok(())
}

fn render_to_dot_string(root_node: &Rc<dyn FeasibleStep>) -> Result<String, RenderError> {
    // ToDo: flatten the whole tree 
    let mut flat_nodes_list: Vec<Rc<dyn FeasibleStep>> = Vec::new();
    flat_nodes_list.push(root_node.clone());
    flat_nodes_list.append(&mut root_node.get_children());

    let mut labels_texts: Vec<String> = Vec::new();
    let mut edges_texts: Vec<String> = Vec::new();

    for node in flat_nodes_list {
        labels_texts.push(format!(r#"{} [{}]"#, node.id(), node.render()));

        if let Some(parent) = node.get_parent() {
            edges_texts.push(format!("{} -> {};", parent.id(), node.id()));
        }
    }

    let dot_content = format!(
        r#"digraph G {{

node [shape=box]

{}

{}

}}"#,
        labels_texts.join("\n"),
        edges_texts.join("\n")
    );

    Ok(dot_content.to_string())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::model::{tests::build_criteria, AndNode, FeasibleStep, Leaf, OrNode};

    use super::render_to_dot_string;

    #[test]
    fn a_single_leaf_can_be_rendered() {
        let definition = build_criteria(&["Kn", "Eq"]);
        let leaf: Rc<dyn FeasibleStep> =
            Rc::new(Leaf::new("Step 1", None, &definition, &[15, 5], || 1));

        let result = render_to_dot_string(&leaf).unwrap();

        let expected = r#"digraph G {

node [shape=box]

1 [label="Step 1\nKn=15, Eq=5"]



}"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn an_and_node_with_a_single_leaf_can_be_rendered() {
        let definition = build_criteria(&["Kn", "Eq"]);

        let root: Rc<dyn FeasibleStep> = Rc::new(AndNode::new("Root", None, || 1));
        let leaf: Rc<dyn FeasibleStep> = Rc::new(Leaf::new(
            "Step 1",
            Some(root.clone()),
            &definition,
            &[15, 5],
            || 2,
        ));
        root.add_child(&leaf);

        let result = render_to_dot_string(&root).unwrap();

        let expected = r#"digraph G {

node [shape=box]

1 [label="Root" shape=trapezium]
2 [label="Step 1\nKn=15, Eq=5"]

1 -> 2;

}"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn an_or_node_with_a_single_leaf_can_be_rendered() {
        let definition = build_criteria(&["Kn", "Eq"]);

        let root: Rc<dyn FeasibleStep> = Rc::new(OrNode::new("Root", None, || 1));
        let leaf: Rc<dyn FeasibleStep> = Rc::new(Leaf::new(
            "Step 1",
            Some(root.clone()),
            &definition,
            &[15, 5],
            || 2,
        ));
        root.add_child(&leaf);

        let result = render_to_dot_string(&root).unwrap();

        let expected = r#"digraph G {

node [shape=box]

1 [label="Root" shape=invtrapezium]
2 [label="Step 1\nKn=15, Eq=5"]

1 -> 2;

}"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn a_multi_level_tree_can_be_rendered() {
        todo!()
    }
}
