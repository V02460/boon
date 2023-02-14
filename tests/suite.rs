use std::{fs::File, path::Path};

use jsonschema::{Compiler, Draft, Schemas, UrlLoader};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SUITE_DIR: &str = "tests/JSON-Schema-Test-Suite";
const TESTS_DIR: &str = "tests/JSON-Schema-Test-Suite/tests";

#[derive(Debug, Serialize, Deserialize)]
struct Group {
    description: String,
    schema: Value,
    tests: Vec<Test>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Test {
    description: String,
    data: Value,
    valid: bool,
}

#[test]
fn test_suite() {
    run_dir("draft6", Draft::V6);
    // run_file("draft4/refRemote.json", Draft::V4);
}

fn run_dir(path: &str, draft: Draft) {
    let path = Path::new(TESTS_DIR).join(path);
    for entry in path.read_dir().unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let entry_path = entry.path();
        let entry_path = entry_path
            .strip_prefix(TESTS_DIR)
            .unwrap()
            .to_str()
            .unwrap();
        if file_type.is_file() {
            run_file(entry_path, draft);
        } else if file_type.is_dir() {
            //run_dir(entry_path, draft);
        }
    }
}

fn run_file(path: &str, draft: Draft) {
    println!("FILE: {path}");
    let path = Path::new(TESTS_DIR).join(path);
    let file = File::open(path).unwrap();

    let url = "http://testsuite.com/schema.json";
    let groups: Vec<Group> = serde_json::from_reader(file).unwrap();
    for group in groups {
        println!("{}", group.description);
        let mut schemas = Schemas::default();
        let mut compiler = Compiler::default();
        compiler.set_default_draft(draft);
        compiler.add_resource(url, group.schema).unwrap();
        compiler.register_url_loader("http", Box::new(RemotesLoader));
        let sch_index = compiler.compile(&mut schemas, url.into()).unwrap();
        for test in group.tests {
            println!("    {}", test.description);
            let result = schemas.validate(&test.data, sch_index);
            if let Err(e) = &result {
                println!("        {e:#}");
            }
            assert_eq!(result.is_ok(), test.valid);
        }
    }
}

struct RemotesLoader;
impl UrlLoader for RemotesLoader {
    fn load(&self, url: &url::Url) -> Result<Value, Box<dyn std::error::Error>> {
        // ensure that url has "localhost:1234"
        if url.as_str().starts_with("http://localhost:1234/") {
            let path = Path::new(SUITE_DIR).join("remotes").join(&url.path()[1..]);
            let file = File::open(path)?;
            let json: Value = serde_json::from_reader(file)?;
            return Ok(json);
        }

        // Meta-Schemas --
        let url = url.as_str();
        let meta = if let Some(suffix) = url.strip_prefix("http://json-schema.org/") {
            Some(suffix)
        } else if let Some(suffix) = url.strip_prefix("https://json-schema.org/") {
            Some(suffix)
        } else {
            None
        };
        if let Some(meta) = meta {
            let file = File::open(Path::new("src/metaschemas/").join(meta))?;
            let json: Value = serde_json::from_reader(file)?;
            return Ok(json);
        }

        Err("no internet")?
    }
}