const BASE_CODE: &str = "
use helix::{Engine, source::Source};
use std::path::Path;

fn run(path: &str, src: &str, expect_error: bool) {
    let mut engine = Engine::new();
    let source = Source {
        content: Box::leak(src.to_string().into_boxed_str()),
        path: Box::leak(Path::new(path).to_path_buf().into_boxed_path()),
    };

    let result = engine.register_program(source).and_then(|_| engine.execute(source));
    if expect_error {
        assert!(result.is_err(), \"expected program to error but it succeeded\");
    } else {
        result.expect(\"program returned an error\");
    }
}
";

fn main() {
    let out = std::env::var("OUT_DIR").unwrap();
    let mut code = String::from(BASE_CODE);

    let entries = std::fs::read_dir("tests/helix")
        .expect("tests/helix directory not found")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "hx").unwrap_or(false));

    for entry in entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(['-', ' '], "_");
        let src = std::fs::read_to_string(&path).unwrap();
        let expect_error = src
            .lines()
            .next()
            .map(|l| l.trim() == "// @error")
            .unwrap_or(false);
        let abs = std::fs::canonicalize(&path).unwrap();

        code.push_str(&format!(
            "#[test]\nfn {name}() {{ run({abs:?}, include_str!({abs:?}), {expect_error}); }}\n"
        ));
    }

    std::fs::write(format!("{out}/helix_tests.rs"), code).unwrap();

    println!("cargo:rerun-if-changed=tests/helix");
}
