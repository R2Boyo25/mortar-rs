use std::process::exit;

use rhai::{EvalAltResult, Engine, Dynamic};
use crate::loader;

type RResult<T> = Result<T, Box<EvalAltResult>>;

pub struct Mortar {
    pub engine: rhai::Engine
}

impl Mortar {
    pub fn new() -> Self {
        Self {
            engine: Self::init_engine().unwrap()
        }
    }

    fn init_engine() -> RResult<Engine> {
        let mut engine = Engine::new();

        engine.set_module_resolver(loader::MortarModuleResolver::new());

        Ok(engine)
    }

    pub fn eval_file(&self, filename: &str) -> Dynamic {
        self.engine.eval_file::<Dynamic>(filename.into()).unwrap_or_else(|error| {
            println!("{}: {}", filename, error);
            exit(1);
        })
    }
}
