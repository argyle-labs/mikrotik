//! Dynamic (subprocess) entrypoint for the mikrotik plugin.
//!
//! Builds the typed `Plugin` and serves it over the orca socket. The plugin is
//! a `[[bin]]`, owns no runtime, and reaches orca only through the socket.
plugin_toolkit::instrument::bootstrap!();
use mikrotik::MikrotikBackend;
use plugin_toolkit::plugin::Plugin;

fn main() -> plugin_toolkit::anyhow::Result<()> {
    Plugin::named("mikrotik")
        .version(env!("CARGO_PKG_VERSION"))
        .service(MikrotikBackend::new("mikrotik"))
        .serve()
}
