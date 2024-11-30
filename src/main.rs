use std::{
    env,
    fs::{self, metadata, DirEntry, File},
    io::{self, BufReader},
    path::PathBuf,
    rc::Rc,
};

use att::{
    model::{FeasibilityCriteria, FeasibleStep, FeasiblityCriterion},
    parser::AttackTreeParser,
    render::render_to_png,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 {
        eprintln!("Usage: att <file or directory name>");
        return Ok(());
    }

    let directory_name = args[0].clone();

    let md = metadata(&directory_name).unwrap();

    if md.is_dir() {
        // parse criteria.json with FeasibilityCriteria
        let definition_file_path = format!("{}/{}", &directory_name, "criteria.json");
        let file_contents = fs::read_to_string(&definition_file_path)
            .expect(&format!("Could not read file {}", &definition_file_path));
        let criteria: Vec<FeasiblityCriterion> =
            serde_json::from_str(&file_contents).expect("criteria file parser error");
        let definition = Rc::new(FeasibilityCriteria(criteria));

        // filter attack tree files
        let paths = fs::read_dir(&directory_name).expect("Error listing files");
        let attack_tree_files: Vec<DirEntry> = paths
            .filter_map(Result::ok)
            .filter(|e| {
                if let Some(e) = e.path().extension() {
                    e == "att"
                } else {
                    false
                }
            })
            .collect();

        // parse attack tree files
        let attack_trees = parse_attack_trees(&attack_tree_files, &definition);

        // render each tree to png
        for (file_path, attack_tree_root) in attack_trees {
            let image_file_path = file_path
                .with_extension("png")
                .to_str()
                .expect("Could not convert target path to str.")
                .to_string();

            render_to_png(&attack_tree_root, &image_file_path)
                .expect(&format!("Error rendering file {}", &image_file_path));
        }
    }

    Ok(())
}

fn parse_attack_trees(
    tree_files: &[DirEntry],
    definition: &Rc<FeasibilityCriteria>,
) -> Vec<(PathBuf, Rc<dyn FeasibleStep>)> {
    let mut steps = vec![];

    for file_entry in tree_files {
        let file_path = file_entry.path();
        let f = File::open(&file_path)
            .expect(&format!("Could not read file {:?}", file_entry.file_name()));
        let mut f = BufReader::new(f);

        let mut parser = AttackTreeParser::new();
        let attack_tree_root = parser
            .parse(&mut f, definition)
            .expect("Error in tree file");
        steps.push((file_path, attack_tree_root));
    }

    steps
}
