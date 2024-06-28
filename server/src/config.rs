#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MysqlConfig {
    pub password: String,
    pub username: String,
    pub dbname: String,
    pub url: String,
}

/// The main configuration of the application
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MainConfiguration {
    pub db: MysqlConfig,
}

/// Load the configuration file from disk
pub fn load_config() -> Result<MainConfiguration, toml::de::Error> {
    let settings_file = std::fs::read_to_string("./server-settings.ini")
        .expect("Failed to open server-settings.ini");
    let settings_result = toml::from_str(&settings_file);
    if let Err(e) = &settings_result {
        println!("Failed to read settings {}", e);
    }
    settings_result
}

/// Open a connection to the mysql server
pub fn open_mysql(settings: &MainConfiguration) -> Result<mysql_async::Pool, mysql_async::Error> {
    let mysql_conn_s = format!(
        "mysql://{}:{}@{}/{}",
        settings.db.username, settings.db.password, settings.db.url, settings.db.dbname
    );
    let mysql_opt = mysql_async::Opts::from_url(mysql_conn_s.as_str())?;
    Ok(mysql_async::Pool::new(mysql_opt))
}