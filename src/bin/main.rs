use std::path::Path;

use helix::{engine::Engine, error, source::Source};

fn main() {
    let mut engine = Engine::new();

    let result = engine.register(Source {
        content: "5 != 2",
        path: Path::new("<stdin>"),
    });

    let source = match result {
        Ok(source) => source,
        Err(e) => {
            error::print_error(e);
            return;
        }
    };

    engine.excecute(source);
}
