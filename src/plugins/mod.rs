mod external;
mod registry;

pub use external::ExternalFormatter;
pub use external::ExternalFormatterMode;
pub(crate) use external::ExternalFormatterOutput;
pub(crate) use external::MissingExecutablePolicy;
pub use registry::PluginRegistry;
pub(crate) use registry::builtin_formatter;
