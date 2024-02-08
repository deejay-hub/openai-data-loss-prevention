use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "action")]
    pub action: String,
    #[serde(alias = "entities")]
    pub entities: Vec<String>,
    #[serde(alias = "language")]
    pub language: String,
    #[serde(
        alias = "presidio_analysis_service",
        deserialize_with = "pdk::serde::deserialize_service"
    )]
    pub presidio_analysis_service: pdk::hl::Service,
    #[serde(alias = "score_threshold")]
    pub score_threshold: f64,
}
#[pdk::hl::entrypoint_flex]
fn init(abi: &dyn pdk::flex_abi::api::FlexAbi) -> Result<(), anyhow::Error> {
    let config: Config = serde_json::from_slice(abi.get_configuration())
        .map_err(|err| {
            anyhow::anyhow!(
                "Failed to parse configuration '{}'. Cause: {}",
                String::from_utf8_lossy(abi.get_configuration()), err
            )
        })?;
    abi.service_create(config.presidio_analysis_service)?;
    Ok(())
}
