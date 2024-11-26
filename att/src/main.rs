use std::{
    env,
    fs::{metadata, File},
    io::{self, BufReader},
    rc::Rc,
};

use att::{
    model::{FeasibilityCriteria, FeasiblityCriterion},
    parser::AttackTreeParser,
    render::render_to_png,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 {
        eprintln!("Usage: att <file or directory name>");
        return Ok(());
    }

    let file_or_directory_name = args[0].clone();

    let md = metadata(&file_or_directory_name).unwrap();

    if md.is_file() {
        
        // ToDo: deserialize this from file
        let definition: Rc<FeasibilityCriteria> = Rc::new(FeasibilityCriteria(vec![
            FeasiblityCriterion {
                name: "Kn".to_string(),
                _id: "1".to_string(),
            },
            FeasiblityCriterion {
                name: "Eq".to_string(),
                _id: "1".to_string(),
            },
            ]));
            
        let f = File::open(&file_or_directory_name)?;
        let mut f = BufReader::new(f);
        let mut parser = AttackTreeParser::new();

        // ToDo: output error line
        let attack_tree_root = parser.parse(&mut f, &definition).expect("Error in tree file");

        let image_file_name = format!("{}.png", &file_or_directory_name);
        render_to_png(&attack_tree_root, &image_file_name).expect(&format!("Error rendering file {}", &image_file_name));
    }


    Ok(())
}
