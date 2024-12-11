use std::io::Result;

fn main() -> Result<()> {
    prost_validate_build::Builder::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        .compile_protos(
            &["../../proto/kurec.proto"],
            &["../../proto/", "../prost-validate-types/proto"],
        )?;
    Ok(())
}
