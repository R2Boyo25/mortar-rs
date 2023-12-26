use rhai::{ModuleResolver, Module, Engine, EvalAltResult, Position, Scope};
use std::{rc::Rc, path::PathBuf};
use normalize_path::NormalizePath;

type RResult<T> = Result<T, Box<EvalAltResult>>;

pub struct MortarModuleResolver {}

impl MortarModuleResolver {
    pub fn new() -> Self {
        Self {
            
        }
    }
    
    fn get_source(file: &str) -> RResult<String> {
        std::fs::read_to_string(
            file,
        ).map_err(|x| Box::new(x.to_string().into()))
    }

    /// TODO
    fn get_path(src: Option<&str>, path: &str) -> String {
        if Self::exists(path) {
            return path.into();
        }
        
        match src {
            Some(src_path) => {
                println!("{src_path}");
                
                PathBuf::from_iter([PathBuf::from(src_path).normalize().to_str().unwrap(), path]).normalize().to_str().unwrap().into()
            },
            None => {
                path.into()
            }
        }
    }
    
    fn load(engine: &Engine, file: &str) -> RResult<Module> {
        let source = Self::get_source(file)?;

        Module::eval_ast_as_new(Scope::new(), &engine.compile(source)?, &engine)
    }

    fn exists(path: &str) -> bool {
        std::fs::metadata(path).is_ok()
    }
}

impl ModuleResolver for MortarModuleResolver {
    fn resolve(
        &self,
        engine: &Engine,
        source_path: Option<&str>,
        path: &str,
        pos: Position,
    ) -> RResult<Rc<Module>> {
        println!("{:?} {} {:?}", source_path, path, pos);
        
        let path = &Self::get_path(source_path, path);
        
        if Self::exists(path) {
            match Self::load(engine, path) {
                Ok(mut module) => {
                    module.build_index();
                    Ok(Rc::new(module))
                },

                Err(err) => Err(EvalAltResult::ErrorInModule(path.into(), err, pos).into())
            }
        } else {
            Err(EvalAltResult::ErrorModuleNotFound(path.into(), pos).into())
        }
    }
}
