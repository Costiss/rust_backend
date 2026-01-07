/// Generate and save OpenAPI spec to JSON file
///
/// This script generates the OpenAPI specification from the application code
/// and saves it to docs/openapi/openapi.json.
///
/// Usage: cargo run --bin generate-openapi
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Import the OpenAPI spec from the library
    use prontua_backend::shared::openapi::ApiDoc;
    use utoipa::OpenApi;

    // Create the output directory if it doesn't exist
    let output_dir = Path::new("docs/openapi");
    fs::create_dir_all(output_dir)?;

    // Generate the OpenAPI spec
    let openapi = ApiDoc::openapi();

    // Serialize to JSON
    let openapi_json = serde_json::to_string_pretty(&openapi)?;

    // Write to file
    let output_path = output_dir.join("openapi.json");
    fs::write(&output_path, openapi_json)?;

    println!("✓ OpenAPI spec generated successfully!");
    println!("  Location: {}", output_path.display());
    println!("  Size: {} bytes", fs::metadata(&output_path)?.len());

    Ok(())
}
