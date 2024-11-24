use std::{
    io::{self, Write},
    process::{Command, Stdio},
};

fn main() -> io::Result<()> {

    let dot_content = r#"
digraph G {

node [shape=box]

node1 [label="Enter house" shape=trapezium]
node2 [label="Observe when people are away" shape=invtrapezium]
node3 [label="Step 1\nKn=15, Eq=5"]
node4 [label="Step 2\nKn=1, Eq=3"]
node5 [label="Step 3\nKn=0, Eq=2"]
node6 [label="Step 4\nKn=4, Eq=0"]
node7 [label="Break into the house" shape=trapezium] 

node1 -> node2;
node1 -> node7;
node2 -> node3;
node2 -> node4;
node7 -> node5;
node7 -> node6;

}"#;

    let mut child = Command::new("dot")
        .args(["-Tpng", "-o tree_img.png"])
        .stdin(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all(dot_content.as_bytes())?;
    

    Ok(())
}
