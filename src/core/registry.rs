use crate::adapters::traits::{ConfirmPolicy, FrameworkAdapter, LanguageAdapter};
use crate::core::errors::{ManscriptError, Result};
use crate::core::runtime::Runtime;
use crate::runtime::RuntimeProvider;
use std::collections::HashMap;

pub struct AdapterRegistry {
    languages: HashMap<String, Box<dyn LanguageAdapter>>,
    frameworks: HashMap<String, Box<dyn FrameworkAdapter>>,
    providers: Vec<Box<dyn RuntimeProvider>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            languages: HashMap::new(),
            frameworks: HashMap::new(),
            providers: Vec::new(),
        }
    }

    pub fn register_language(&mut self, adapter: Box<dyn LanguageAdapter>) {
        self.languages.insert(adapter.id().to_string(), adapter);
    }

    pub fn register_framework(&mut self, adapter: Box<dyn FrameworkAdapter>) {
        self.frameworks.insert(adapter.id().to_string(), adapter);
    }

    pub fn register_provider(&mut self, provider: Box<dyn RuntimeProvider>) {
        self.providers.push(provider);
    }

    pub fn language(&self, id: &str) -> Result<&dyn LanguageAdapter> {
        self.languages
            .get(id)
            .map(|a| a.as_ref())
            .ok_or_else(|| ManscriptError::UnknownLanguage(id.to_string()))
    }

    pub fn framework(&self, id: &str) -> Result<&dyn FrameworkAdapter> {
        self.frameworks
            .get(id)
            .map(|a| a.as_ref())
            .ok_or_else(|| ManscriptError::UnknownFramework(id.to_string()))
    }

    pub fn frameworks(&self) -> Vec<&dyn FrameworkAdapter> {
        let mut v: Vec<_> = self.frameworks.values().map(|b| b.as_ref()).collect();
        v.sort_by_key(|f| f.id());
        v
    }

    pub fn languages(&self) -> Vec<&dyn LanguageAdapter> {
        let mut v: Vec<_> = self.languages.values().map(|b| b.as_ref()).collect();
        v.sort_by_key(|l| l.id());
        v
    }

    pub fn providers(&self) -> &[Box<dyn RuntimeProvider>] {
        &self.providers
    }

    pub fn frameworks_for_language(&self, language: &str) -> Vec<&dyn FrameworkAdapter> {
        self.frameworks()
            .into_iter()
            .filter(|f| f.language() == language && !f.language_only())
            .collect()
    }

    pub fn default_provider_id(language: &str) -> &'static str {
        match language {
            "python" => "uv",
            "ruby" | "go" | "rust" | "php" | "csharp" => "mise",
            _ => "system",
        }
    }

    pub fn resolve_runtime(
        &self,
        language: &str,
        version: &str,
        preferred_provider: Option<&str>,
        confirm: ConfirmPolicy,
    ) -> Result<Runtime> {
        if let Some(id) = preferred_provider {
            let provider = self
                .providers
                .iter()
                .find(|p| p.id() == id)
                .ok_or_else(|| {
                    ManscriptError::Message(format!(
                        "`{id}` is not a registered runtime provider.\n\nCheck `[runtime].provider` in `manscript.toml`, or remove that setting to use the default provider."
                    ))
                })?;
            if !provider.supports(language) {
                return Err(ManscriptError::Message(format!(
                    "Runtime provider `{id}` does not support {language}.\n\nChoose a compatible provider in `manscript.toml`, or remove `[runtime].provider` to use the default."
                )));
            }
            return provider.prepare(language, version, confirm);
        }

        if let Some(system) = self.providers.iter().find(|p| p.id() == "system") {
            if let Some(runtime) = system.detect(language, version)? {
                return Ok(runtime);
            }
        }

        let default_id = Self::default_provider_id(language);
        if let Some(provider) = self.providers.iter().find(|p| p.id() == default_id) {
            if provider.supports(language) {
                return provider.prepare(language, version, confirm);
            }
        }

        Err(ManscriptError::RuntimeNotFound {
            language: language.to_string(),
            version: version.to_string(),
        })
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_registry() -> AdapterRegistry {
    let mut registry = AdapterRegistry::new();
    registry.register_language(Box::new(crate::adapters::python::PythonAdapter));
    registry.register_language(Box::new(crate::adapters::ruby::RubyAdapter));
    registry.register_language(Box::new(crate::adapters::c::CAdapter));
    registry.register_language(Box::new(crate::adapters::cpp::CppAdapter));
    registry.register_language(Box::new(crate::adapters::java::JavaAdapter));
    registry.register_language(Box::new(crate::adapters::go::GoAdapter));
    registry.register_language(Box::new(crate::adapters::rust::RustAdapter));
    registry.register_language(Box::new(crate::adapters::php::PhpAdapter));
    registry.register_language(Box::new(crate::adapters::csharp::CsharpAdapter));
    registry.register_framework(Box::new(
        crate::adapters::python::frameworks::plain::PlainPythonFramework,
    ));
    registry.register_framework(Box::new(
        crate::adapters::ruby::frameworks::plain::PlainRubyFramework,
    ));
    registry.register_framework(Box::new(crate::adapters::c::plain::PlainCFramework));
    registry.register_framework(Box::new(crate::adapters::cpp::plain::PlainCppFramework));
    registry.register_framework(Box::new(crate::adapters::java::plain::PlainJavaFramework));
    registry.register_framework(Box::new(crate::adapters::go::plain::PlainGoFramework));
    registry.register_framework(Box::new(crate::adapters::rust::plain::PlainRustFramework));
    registry.register_framework(Box::new(crate::adapters::php::plain::PlainPhpFramework));
    registry.register_framework(Box::new(
        crate::adapters::csharp::plain::PlainCsharpFramework,
    ));
    registry.register_framework(Box::new(
        crate::adapters::python::frameworks::django::DjangoFramework,
    ));
    registry.register_framework(Box::new(
        crate::adapters::python::frameworks::fastapi::FastApiFramework,
    ));
    registry.register_framework(Box::new(
        crate::adapters::python::frameworks::flask::FlaskFramework,
    ));
    registry.register_framework(Box::new(
        crate::adapters::ruby::frameworks::rails::RailsFramework,
    ));
    registry.register_framework(Box::new(
        crate::adapters::ruby::frameworks::sinatra::SinatraFramework,
    ));
    registry.register_provider(Box::new(crate::runtime::system::SystemRuntimeProvider));
    registry.register_provider(Box::new(crate::runtime::uv::UvPythonProvider));
    registry.register_provider(Box::new(crate::runtime::mise::MiseProvider));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn django_maps_to_python() {
        let r = default_registry();
        let fw = r.framework("django").unwrap();
        assert_eq!(fw.language(), "python");
    }

    #[test]
    fn python_language_only_adapter() {
        let r = default_registry();
        let fw = r.framework("python").unwrap();
        assert!(fw.language_only());
        assert_eq!(fw.language(), "python");
        assert!(r
            .frameworks_for_language("python")
            .iter()
            .all(|f| !f.language_only()));
        let ruby = r.framework("ruby").unwrap();
        assert!(ruby.language_only());
        assert_eq!(ruby.language(), "ruby");
        assert!(r
            .frameworks_for_language("ruby")
            .iter()
            .all(|f| !f.language_only()));
    }

    #[test]
    fn in_project_generators_are_adapter_owned() {
        let r = default_registry();
        let django = r.framework("django").unwrap();
        assert_eq!(django.generators().len(), 2);
        assert_eq!(django.generators()[0].id, "app");
        assert!(django.generators().iter().any(|g| g.id == "model"));
        let rails = r.framework("rails").unwrap();
        assert!(rails.generators().iter().any(|g| g.id == "scaffold"));
        assert!(r.framework("python").unwrap().generators().is_empty());
        assert!(r.framework("c").unwrap().language_only());
        assert!(r.framework("cpp").unwrap().language_only());
        assert!(r.framework("java").unwrap().language_only());
        assert!(r.framework("go").unwrap().language_only());
        assert!(r.framework("rust").unwrap().language_only());
        assert!(r.framework("php").unwrap().language_only());
        assert!(r.framework("csharp").unwrap().language_only());
        assert!(r.frameworks_for_language("c").is_empty());
        assert!(r.frameworks_for_language("java").is_empty());
        assert!(r.framework("flask").unwrap().generators()[0].id == "blueprint");
        assert!(r.framework("fastapi").unwrap().generators()[0].id == "router");
        assert!(r.framework("sinatra").unwrap().generators()[0].id == "routes");
    }

    #[test]
    fn new_languages_use_managed_fallbacks() {
        for language in ["go", "rust", "php", "csharp"] {
            assert_eq!(AdapterRegistry::default_provider_id(language), "mise");
        }
        assert_eq!(AdapterRegistry::default_provider_id("java"), "system");
    }
}
