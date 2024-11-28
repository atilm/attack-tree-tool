use std::{
    env,
    fs::{self, metadata, DirEntry, File},
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

    let directory_name = args[0].clone();

    let md = metadata(&directory_name).unwrap();

    if md.is_dir() {
        // let definition = read_criteria_definition(&directory_name);

        // ToDo: deserialize this from file
        let definition: Rc<FeasibilityCriteria> = Rc::new(FeasibilityCriteria(vec![
            FeasiblityCriterion {
                name: "Knowledge".to_string(),
                id: "Kn".to_string(),
            },
            FeasiblityCriterion {
                name: "Equipment".to_string(),
                id: "Eq".to_string(),
            },
        ]));

        let paths = fs::read_dir(&directory_name).expect("Error listing files");

        let attack_tree_files: Vec<DirEntry> = paths
            .filter_map(Result::ok)
            .filter(|e| if let Some(e) = e.path().extension() { e == "att"} else { false })
            .collect();

        // render each file to png
        for file_entry in attack_tree_files {
            let file_path = file_entry.path();
            let f = File::open(&file_path)?;
            let mut f = BufReader::new(f);

            let mut parser = AttackTreeParser::new();
            let attack_tree_root = parser
                .parse(&mut f, &definition)
                .expect("Error in tree file");

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
