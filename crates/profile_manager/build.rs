fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../proto/acolors.proto")?;
    let builder = tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute(
            "v2ray.config.OutboundObject.OutboundSettings.kind",
            "#[serde(flatten)]",
        )
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    builder.compile(&["../../proto/v2ray_config.proto"], &["../../proto"])?;
    Ok(())
}
