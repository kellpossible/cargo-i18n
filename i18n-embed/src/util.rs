/// Get the translation domain from the module path (first module in
/// the module path).
pub fn domain_from_module(module_path: &str) -> &str {
    module_path.split("::").next().unwrap_or(module_path)
}
