use std::rc::Rc;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use thiserror::Error;
use markdown_table_formatter::format_tables;

use crate::model::feasible_step::FeasibleStep;

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
    let mut flat_nodes_list: Vec<Rc<dyn FeasibleStep>> = Vec::new();
    flatten(root_node, &mut flat_nodes_list);

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

fn flatten(node: &Rc<dyn FeasibleStep>, result: &mut Vec<Rc<dyn FeasibleStep>>) {
    result.push(node.clone());

    for c in node.get_children() {
        flatten(&c, result);
    }
}

pub fn render_to_markdown_table(root_nodes: Vec<&Rc<dyn FeasibleStep>>) -> String {
    let mut result = "| Threat Id | Threat | Feasbility | Impact | Risk |\n".to_string();
    result.push_str("|--|--|--|--|--|\n");

    for node in root_nodes {
        result.push_str(&format!("| {} | {} | {} | | |\n", node.id(), node.title(), node.feasibility_value()));
    }

    format_tables(result)
}

#[cfg(test)]
mod tests {
    use crate::model::feasible_step::FeasibleStep;
    use std::rc::Rc;

    use crate::model::{tests::build_criteria, AndNode, Leaf, OrNode};

    use super::render_to_dot_string;

    #[test]
    fn a_single_leaf_can_be_rendered() {
        let definition = build_criteria(&["Kn", "Eq"]);
        let leaf: Rc<dyn FeasibleStep> =
            Rc::new(Leaf::new("Step 1", None, &definition, &[15, 5], || 1));

        let result = render_to_dot_string(&leaf).unwrap();

        let expected = r#"digraph G {

node [shape=box]

1 [label="Step 1\n20\nKn=15, Eq=5"]



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

1 [label="Root\n20\nKn=15, Eq=5" shape=trapezium]
2 [label="Step 1\n20\nKn=15, Eq=5"]

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

1 [label="Root\n20\nKn=15, Eq=5" shape=invtrapezium]
2 [label="Step 1\n20\nKn=15, Eq=5"]

1 -> 2;

}"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn a_multi_level_tree_can_be_rendered() {
        let definition = build_criteria(&["Kn", "Eq"]);

        let tree: Rc<dyn FeasibleStep> = Rc::new(AndNode::new("Root", None, || 1));

        let first_subtree: Rc<dyn FeasibleStep> =
            Rc::new(AndNode::new("First Sub", Some(tree.clone()), || 2));
        tree.add_child(&first_subtree);
        let leaf1: Rc<dyn FeasibleStep> = Rc::new(Leaf::new(
            "Leaf 1",
            Some(first_subtree.clone()),
            &definition,
            &[1, 5],
            || 3,
        ));
        let leaf2: Rc<dyn FeasibleStep> = Rc::new(Leaf::new(
            "Leaf 2",
            Some(first_subtree.clone()),
            &definition,
            &[3, 1],
            || 4,
        ));
        first_subtree.add_child(&leaf1);
        first_subtree.add_child(&leaf2);

        let second_subtree: Rc<dyn FeasibleStep> =
            Rc::new(OrNode::new("Second Sub", Some(tree.clone()), || 5));
        tree.add_child(&second_subtree);
        let leaf3: Rc<dyn FeasibleStep> = Rc::new(Leaf::new(
            "Leaf 3",
            Some(second_subtree.clone()),
            &definition,
            &[2, 14],
            || 6,
        ));
        let leaf4: Rc<dyn FeasibleStep> = Rc::new(Leaf::new(
            "Leaf 4",
            Some(second_subtree.clone()),
            &definition,
            &[20, 1],
            || 7,
        ));
        second_subtree.add_child(&leaf3);
        second_subtree.add_child(&leaf4);

        let result = render_to_dot_string(&tree).unwrap();

        let expected = r#"digraph G {

node [shape=box]

1 [label="Root\n17\nKn=3, Eq=14" shape=trapezium]
2 [label="First Sub\n8\nKn=3, Eq=5" shape=trapezium]
3 [label="Leaf 1\n6\nKn=1, Eq=5"]
4 [label="Leaf 2\n4\nKn=3, Eq=1"]
5 [label="Second Sub\n16\nKn=2, Eq=14" shape=invtrapezium]
6 [label="Leaf 3\n16\nKn=2, Eq=14"]
7 [label="Leaf 4\n21\nKn=20, Eq=1"]

1 -> 2;
2 -> 3;
2 -> 4;
1 -> 5;
5 -> 6;
5 -> 7;

}"#;
        assert_eq!(result, expected);
    }
}
