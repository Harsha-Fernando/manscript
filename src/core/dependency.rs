#[derive(Debug, Clone)]
pub struct DependencySpec {
    pub name: String,
    pub version: Option<String>,
}
