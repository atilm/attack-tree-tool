use std::{
    env,
    ffi::OsStr,
    fs::{self, metadata, DirEntry, File},
    io::BufReader,
    path::{Path, PathBuf},
    process::exit,
    rc::Rc,
};

use att::{
    model::{feasible_step::FeasibleStep, FeasibilityCriteria, FeasiblityCriterion},
    parser::AttackTreeParser,
    render::render_to_markdown_table,
    render::render_to_png,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 {
        eprintln!("Usage: att <file or directory name>");
        exit(1);
    }

    let directory_name = args[0].clone();

    let md = match metadata(&directory_name) {
        Ok(m) => m,
        Err(e) => {
            println!("{}: {}", e, directory_name);
            exit(1);
        }
    };

    if !md.is_dir() {
        println!("'{}' is not a directory.", &directory_name);
        exit(1);
    }

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

    let images_dir = Path::new("images");
    let absolute_images_dir = Path::new(&directory_name).join(images_dir);
    if fs::create_dir_all(&absolute_images_dir).is_err() {
        println!("Could not create {:?}", &absolute_images_dir)
    }

    // render each tree to png
    for (file_path, attack_tree_root) in &attack_trees {
        let image_file_path = &to_image_path(&absolute_images_dir, file_path);
        render_to_png(&attack_tree_root, image_file_path)
            .expect(&format!("Error rendering file {:?}", image_file_path));
    }

    // render to markdown overview file
    let threats_file_path = format!("{}/threats.md", directory_name);

    let root_nodes: Vec<_> = attack_trees
        .iter()
        .map(|(f, r)| (to_image_path(images_dir, f), r))
        .collect();

    if let Err(e) = fs::write(&threats_file_path, render_to_markdown_table(root_nodes)) {
        println!("Error writing file {}: {}", &threats_file_path, e);
    }
}

fn to_image_path(images_dir: &Path, attack_tree_path: &PathBuf) -> PathBuf {
    
    images_dir.join(
        Path::new(attack_tree_path.file_name().unwrap_or(OsStr::new("image")))
            .with_extension("png"),
    )
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
