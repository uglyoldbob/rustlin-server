/// The main configuration of the application
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub window: bool,
    pub game_resources: String,
}
