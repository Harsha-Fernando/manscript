use crate::cli::run;

pub fn execute(
    registry: &crate::core::registry::AdapterRegistry,
    args: &[String],
) -> crate::core::errors::Result<()> {
    run::execute_build(registry, args)
}
