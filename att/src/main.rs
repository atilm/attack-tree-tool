use std::{
    io::{self},
    rc::Rc,
};

use att::{
    model::{generate_id, AndNode, FeasibleStep},
    render::render_to_png,
};

fn main() -> io::Result<()> {
    let root_node: Rc<dyn FeasibleStep> = Rc::new(AndNode::new("title", None, generate_id));

    render_to_png(&root_node, "tree_img.png").expect("error");

    Ok(())
}
