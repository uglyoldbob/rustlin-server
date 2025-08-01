use std::io::Read;

/// The configuration needed to make a connection to a mysql server
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MysqlConfig {
    /// The password for the mysql login
    pub password: String,
    /// The username to login to the mysql server with
    pub username: String,
    /// The name of the database to use
    pub dbname: String,
    /// The url of where the mysql server is located
    pub url: String,
}

/// The main server configuration
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ServerConfiguration {
    /// Should new accounts be automatically created if they don't exist? This is probably best for test servers only.
    pub automatic_account_creation: bool,
    /// The salt for account creation
    pub account_creation_salt: String,
}

impl ServerConfiguration {
    /// Get the news for the server, dynamically
    pub fn get_news(&self) -> String {
        let f = std::fs::File::open("./news.txt");
        if let Ok(mut f) = f {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                s
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}

/// The main configuration of the application
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MainConfiguration {
    /// The database configuration
    pub db: MysqlConfig,
    /// The main server configuration
    pub config: ServerConfiguration,
}

/// Load the configuration file from disk
pub fn load_config() -> Result<MainConfiguration, toml::de::Error> {
    let settings_file = std::fs::read_to_string("./server-settings.ini")
        .expect("Failed to open server-settings.ini");
    let settings_result = toml::from_str(&settings_file);
    if let Err(e) = &settings_result {
        log::error!("Failed to read settings {}", e);
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
