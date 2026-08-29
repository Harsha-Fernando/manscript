use crate::adapters::traits::ConfirmPolicy;
use crate::core::errors::Result;
use crate::core::runtime::Runtime;

pub mod download;
pub mod mise;
pub mod system;
pub mod uv;

pub trait RuntimeProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn supports(&self, language: &str) -> bool;
    fn detect(&self, language: &str, version: &str) -> Result<Option<Runtime>>;
    fn prepare(&self, language: &str, version: &str, confirm: ConfirmPolicy) -> Result<Runtime>;
}
