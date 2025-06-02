use lsb_core::{hash::Hash, *};

const INPUT: &[u8] = include_bytes!("../../data/input.webp");
const CONTAINER: &[u8] = include_bytes!("../../data/container.webp");
const EMBEDDED: &[u8] = include_bytes!("../../data/embedded.png");

#[test]
fn test_embed() {
    let hash = Hash::Sha256;
    let seed = 42;
    let lsbs = 1;
    let format = image::ImageFormat::WebP;

    let result = embed(INPUT, "webp", CONTAINER, lsbs, hash, seed, format);

    assert!(result.is_ok(), "Failed to embed data: {:?}", result.err());
}

#[test]
fn test_extract() {
    let seed = 42;
    let lsbs = 1;

    let extracted_result = extract(EMBEDDED, lsbs, seed);

    assert!(
        extracted_result.is_ok(),
        "Failed to extract data: {:?}",
        extracted_result.err()
    );
}

#[test]
fn test_embed_extract() -> Result<(), Box<dyn std::error::Error>> {
    let hash = Hash::Sha256;
    let seed = 42;
    let lsbs = 1;
    let format = image::ImageFormat::WebP;

    let embedded_result = embed(INPUT, "webp", CONTAINER, lsbs, hash, seed, format);

    assert!(
        embedded_result.is_ok(),
        "Failed to embed data: {:?}",
        embedded_result.err()
    );

    let embedded_data = embedded_result?;

    let extracted_result = extract(&embedded_data, lsbs, seed);

    assert!(
        extracted_result.is_ok(),
        "Failed to extract data: {:?}",
        extracted_result.err()
    );

    let extracted_data = extracted_result?.0;

    assert_eq!(
        INPUT, &extracted_data,
        "Extracted data does not match input"
    );

    Ok(())
}
