use crate::{InnerModuleDescriptor, Loader, LoaderError, LoaderResult, ModuleDescriptor};
use async_trait::async_trait;
use gor_ast::module::SourceModule;
use gor_ast::name::Name;
use gor_ast::Parseable;
use gor_parse::{parse, Rule};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone)]
pub struct FileLoader {
    file: PathBuf,
}

impl FileLoader {
    pub fn new<T: Into<PathBuf>>(file: T) -> Self {
        Self { file: file.into() }
    }
    fn load_from_string(input: String, module: Name) -> LoaderResult<ModuleDescriptor> {
        let descriptor = InnerModuleDescriptor::try_new(input, |input| {
            parse(Rule::module, input).map_or_else(
                |e| Err(LoaderError::ParseError(e)),
                |p| SourceModule::parse(p).map_err(Into::into),
            )
        })?;
        if descriptor.borrow_dependent().package == module {
            Ok(ModuleDescriptor(descriptor))
        } else {
            Err(LoaderError::ModuleNotFound(module))
        }
    }
}

#[async_trait]
impl Loader for FileLoader {
    async fn load_module(&self, module: Name) -> LoaderResult<ModuleDescriptor> {
        let mut input = String::new();

        let mut path = None;
        if module == "main".into() {
            path = Some(self.file.to_path_buf());
        }
        if let Some(parent) = self.file.parent() {
            let mod_file = parent.join(module).with_extension(".go");
            if mod_file.exists() {
                path = Some(mod_file)
            } else {
                let mod_dir = parent.join(module).join("mod.go");
                if mod_dir.exists() {
                    path = Some(mod_dir)
                }
            }
        }
        let path = path.ok_or(LoaderError::ModuleNotFound(module))?;

        let mut file = File::open(path).await?;
        file.read_to_string(&mut input).await?;
        FileLoader::load_from_string(input, module)
    }
}
