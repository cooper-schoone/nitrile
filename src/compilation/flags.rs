/// A conditional-compilation flag passed through [`EngineArgs`] to the LaTeX engine, which
/// materializes it as a `nitrile@arg@<key>` macro consumed by `nitrile.sty` at compile time.
///
/// [`EngineArgs`]: crate::compilation::engine::EngineArgs
#[derive(Debug, Eq, PartialEq)]
pub enum Flag {
    String { key: String, value: String },
    Boolean { key: String, value: bool },
}
