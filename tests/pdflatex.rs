use std::path::Path;

use nitrile::compilation::{
    engine::{EngineArgs, LatexEngine},
    flags::Flag,
    pdflatex::PdflatexEngine,
};
use nitrile::sys::environment::SystemEnvironment;

use color_eyre::Result;
use color_eyre::eyre::ensure;

#[test_with::executable(pdflatex)]
#[test]
fn test_pdflatex_engine_compiles_document_correctly_without_flags() -> Result<()> {
    let output_dir = tempfile::tempdir_in("tests")?;

    let result: Result<()> = (|| {
        let output_path = output_dir.path().join("output.pdf");
        let target = Path::new("tests/test_document.tex").to_path_buf();
        let engine = PdflatexEngine {
            environment: SystemEnvironment,
        };

        let computed_output = engine.compile(EngineArgs {
            target,
            output: output_path.clone(),
            flags: vec![],
            verbose: false,
        })?;
        ensure!(
            computed_output == output_path,
            "computed output path did not match the requested output path"
        );

        let text = pdf_extract::extract_text(&computed_output)?;
        ensure!(
            text.contains("Expected document content"),
            "document body was not rendered"
        );
        ensure!(
            text.contains("Anonymous"),
            "unset name flag should render the default \"Anonymous\""
        );
        ensure!(
            text.contains("Conditionally rendered content"),
            "conditional content should render when its flag is unset"
        );
        ensure!(
            !text.contains("Name flag detected"),
            "name-flag branch should not render when the flag is unset"
        );
        Ok(())
    })();

    output_dir.close()?;
    result
}

#[test_with::executable(pdflatex)]
#[test]
fn test_pdflatex_engine_compiles_document_correctly_with_flags() -> Result<()> {
    let output_dir = tempfile::tempdir_in("tests")?;

    let result: Result<()> = (|| {
        let output_path = output_dir.path().join("output.pdf");
        let target = Path::new("tests/test_document.tex").to_path_buf();
        let engine = PdflatexEngine {
            environment: SystemEnvironment,
        };

        let computed_output = engine.compile(EngineArgs {
            target,
            output: output_path.clone(),
            flags: vec![
                Flag::Boolean {
                    key: "show-conditional-content".to_string(),
                    value: false,
                },
                Flag::String {
                    key: "name".to_string(),
                    value: "John Doe".to_string(),
                },
            ],
            verbose: false,
        })?;
        ensure!(
            computed_output == output_path,
            "computed output path did not match the requested output path"
        );

        let text = pdf_extract::extract_text(&computed_output)?;
        ensure!(
            text.contains("Expected document content"),
            "document body was not rendered"
        );
        ensure!(
            text.contains("John Doe"),
            "name flag value was not rendered"
        );
        ensure!(
            !text.contains("Anonymous"),
            "default \"Anonymous\" should not render when the name flag is set"
        );
        ensure!(
            !text.contains("Conditionally rendered content"),
            "conditional content should not render when its flag is false"
        );
        ensure!(
            text.contains("Name flag detected"),
            "name-flag branch should render when the flag is set"
        );
        Ok(())
    })();

    output_dir.close()?;
    result
}
