use std::path::Path;

use nitrile::{
    commands::flags::Flag,
    compilation::{
        engine::{EngineArgs, LatexEngine},
        pdflatex::PdflatexEngine,
    },
};

use color_eyre::Result;

#[test_with::executable(pdflatex)]
#[test]
fn test_pdflatex_engine_compiles_document_correctly_without_flags() -> Result<()> {
    let output_dir = tempfile::tempdir_in("tests")?;

    let result: Result<()> = (|| {
        let output_path = output_dir.path().join("output.pdf");
        let target = Path::new("tests/test_document.tex").to_path_buf();
        let engine = PdflatexEngine {};

        let computed_output = engine.compile(EngineArgs {
            target,
            output: output_path.clone(),
            flags: vec![],
            verbose: false,
        })?;
        assert_eq!(computed_output, output_path);

        let text = pdf_extract::extract_text(&computed_output)?;
        assert!(text.contains("Expected document content"));
        assert!(text.contains("Anonymous"));
        assert!(text.contains("Conditionally rendered content"));
        assert!(!text.contains("Name flag detected"));
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
        let engine = PdflatexEngine {};

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
        assert_eq!(computed_output, output_path);

        let text = pdf_extract::extract_text(&computed_output)?;
        assert!(text.contains("Expected document content"));
        assert!(text.contains("John Doe"));
        assert!(!text.contains("Anonymous"));
        assert!(!text.contains("Conditionally rendered content"));
        assert!(text.contains("Name flag detected"));
        Ok(())
    })();

    output_dir.close()?;
    result
}
