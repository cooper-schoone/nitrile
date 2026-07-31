use std::path::Path;

use nitrile::compilation::{
    engine::{EngineArgs, LatexEngine},
    pdflatex::PdflatexEngine,
};

use color_eyre::Result;

#[test_with::executable(pdflatex)]
#[test]
fn test_pdflatex_engine_compiles_document_correctly() -> Result<()> {
    let output_dir = tempfile::tempdir_in("tests")?;
    let output_path = output_dir.path().join("output.pdf");
    let target = Path::new("tests/test_document.tex");
    let engine = PdflatexEngine {};

    let computed_output = engine.compile(EngineArgs {
        target,
        output: Some(output_path.as_path()),
    })?;
    assert_eq!(computed_output, output_path);

    let text = pdf_extract::extract_text(&computed_output)?;
    assert!(text.contains("Expected document content"));

    output_dir.close()?;
    Ok(())
}
