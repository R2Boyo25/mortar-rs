use std::collections::HashMap;
use std::process::exit;
use normalize_path::NormalizePath;

use starlark::environment::{GlobalsBuilder, Module, FrozenModule};
use starlark::eval::Evaluator;
use starlark::starlark_module;
use starlark::syntax::{AstModule, Dialect};
use starlark::values::{Heap, Value};

pub struct Mortar {
    globals: starlark::environment::Globals,
}

#[starlark_module]
fn globals(builder: &mut GlobalsBuilder) {
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

    fn get_source(relative_to: &str, file: &str) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(std::path::PathBuf::from_iter(vec![relative_to, file]).normalize())?)
    }

    fn loader_from_ast(&self, relative_to: &str, ast: &starlark::syntax::AstModule) -> anyhow::Result<HashMap<String, FrozenModule>> {
        let mut loads = Vec::new();
        for load in ast.loads() {
            loads.push((load.module_id.to_owned(), self.get_module(relative_to, load.module_id)?));
        }
        Ok(loads.iter().map(|(a, b)| (a.as_str().to_owned(), b.to_owned())).collect())
    }

    fn borrowed_modules<'a>(modules: &'a HashMap<String, FrozenModule>) -> HashMap<&str, &FrozenModule> {
         modules.iter().map(|(a, b)| (a.as_str(), b)).collect::<HashMap<&str, &FrozenModule>>()
    }
    
    fn get_module(&self, relative_to: &str, file: &str) -> anyhow::Result<FrozenModule> {
        let ast = AstModule::parse(file, Self::get_source(relative_to, file)?, &Dialect::Standard)?;

        let modules = self.loader_from_ast(relative_to, &ast)?;
        let mut loader = starlark::eval::ReturnFileLoader {modules: &Self::borrowed_modules(&modules) };

        let module = Module::new();
        {
            let mut eval = Evaluator::new(&module);
            eval.set_loader(&mut loader);
            eval.eval_module(ast, &self.globals)?;
        }

        Ok(module.freeze()?)
    }
    
    fn eval_internal(&self, cwd: &str, filename: &str, content: &str) -> anyhow::Result<i8> {
        let ast: AstModule = AstModule::parse(
            filename,
            content.to_owned(),
            &Dialect::Standard,
        )?;

        let normalized_filename = std::path::PathBuf::from_iter(vec![cwd, filename]).normalize();
        let file_dir = normalized_filename.parent().unwrap().to_str().unwrap();
        let modules = self.loader_from_ast(file_dir, &ast)?;
        let mut loader = starlark::eval::ReturnFileLoader {modules: &Self::borrowed_modules(&modules) };
        let module: Module = Module::new();

        module.set("cwd", module.heap().alloc_str(cwd).to_value());
        module.set("current_file", module.heap().alloc_str(filename).to_value());

        let mut eval: Evaluator = Evaluator::new(&module);

        eval.set_loader(&mut loader);
        eval.eval_module(ast, &self.globals)?;

        Ok(0)
    }
    
    pub fn eval<S: AsRef<str>>(&self, cwd: S, filename: S, content: S) {
        self.eval_internal(cwd.as_ref(), filename.as_ref(), content.as_ref()).unwrap_or_else(|error| {
                println!("{}", error);
                exit(1);
            });
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
        ).unwrap_or_else(|error| {
                println!("{}", error);
                exit(1);
            });
    }
}

#[macro_export]
macro_rules! embedded_eval {
    ($mortar: expr, $cwd: expr, $filename: expr) => {
        $mortar.eval($cwd, $filename, include_str!($filename));
    };
}

pub use embedded_eval;
