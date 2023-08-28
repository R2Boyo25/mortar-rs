use std::process::exit;

use starlark::environment::{GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::starlark_module;
use starlark::syntax::{AstModule, Dialect};
use starlark::values::{Heap, Value};

pub struct Mortar {
    globals: starlark::environment::Globals,
}

#[starlark_module]
fn globals(builder: &mut GlobalsBuilder) {
    fn register_<'v>(heap: &'v Heap) -> anyhow::Result<Value> {
        Ok(heap.alloc_str("HEWWO").to_value())
    }

    fn do_something_else<'v>(heap: &'v Heap) -> anyhow::Result<Value> {
        Ok(heap.alloc_str("potato").to_value())
    }
}

impl Mortar {
    pub fn new() -> Self {
        Self {
            globals: Self::create_globals(),
        }
    }

    fn create_globals() -> starlark::environment::Globals {
        GlobalsBuilder::extended().with(globals).build()
    }

    fn eval_internal(&self, cwd: &str, filename: &str, content: &str) {
        let ast: AstModule = AstModule::parse(
            filename,
            content.to_owned(),
            &Dialect::Standard,
        )
        .unwrap();

        let module: Module = Module::new();

        module.set("cwd", module.heap().alloc_str(cwd).to_value());

        let mut eval: Evaluator = Evaluator::new(&module);

        eval.eval_module(ast, &self.globals)
            .unwrap_or_else(|error| {
                println!("{}", error);
                exit(1);
            });
    }
    
    pub fn eval<S: AsRef<str>>(&self, cwd: S, filename: S, content: S) {
        self.eval_internal(cwd.as_ref(), filename.as_ref(), content.as_ref());
    }

    pub fn eval_file<S: AsRef<str>>(&self, cwd: S, filename: S) {
        self.eval_internal(
            cwd.as_ref(),
            filename.as_ref(),
            &std::fs::read_to_string(filename.as_ref()).expect(&format!(
                "{}: File `{}` does not exist.",
                std::env::args().nth(0).unwrap(),
                filename.as_ref()
            )),
        );
    }
}

#[macro_export]
macro_rules! embedded_eval {
    ($mortar: expr, $cwd: expr, $filename: expr) => {
        $mortar.eval($cwd, $filename, include_str!($filename));
    };
}

pub use embedded_eval;
