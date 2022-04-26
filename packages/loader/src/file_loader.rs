use crate::{InnerModuleDescriptor, Loader, LoaderError, LoaderResult, ModuleDescriptor};
use async_trait::async_trait;
use gor_ast::module::Module;
use gor_parse::{parse, Rule};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct FileLoader {
    file: PathBuf,
}

impl FileLoader {
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }
}

#[async_trait]
impl Loader for FileLoader {
    async fn load_module(&self, module: &str) -> LoaderResult<ModuleDescriptor> {
        let mut input = String::new();
        let mut file = File::open(&self.file).await?;
        file.read_to_string(&mut input).await?;
        let descriptor = InnerModuleDescriptor::try_new(input, |input| {
            parse(Rule::module, input).map_or_else(
                |e| Err(LoaderError::ParseError(e)),
                |p| Module::try_from(p).map_err(Into::into),
            )
        })?;
        if descriptor.with_dependent(|_, m| m.package) == module.into() {
            Ok(ModuleDescriptor(descriptor))
        } else {
            Err(LoaderError::ModuleNotFound(module.into()))
        }
    }
}
