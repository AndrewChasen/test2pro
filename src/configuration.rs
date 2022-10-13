use secrecy::Secret;
use secrecy::ExposeSecret;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use sqlx::ConnectOptions;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database:DatabaseSettings,
    pub application:ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port:u16,
    pub host:String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port:u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    // pub fn connection_string(&self)->Secret<String> {
    //     Secret::new(format!(
    //         "postgres://{}:{}@{}:{}/{}",
    //         self.username, self.password.expose_secret(), self.host, self.port, self.database_name
    //     ))
    // }

    pub fn with_db(&self)->PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
        // self.without_db().database(&self.database_name)
    }

    // pub fn connection_string_without_db(&self) ->Secret<String>{
    //     Secret::new(format!(
    //         "postgres://{}:{}@{}:{}",
    //         self.username, self.password.expose_secret(), self.host, self.port
    //     ))

    // }

    pub fn without_db(&self) ->PgConnectOptions {
        let ssl_mode = if self.require_ssl{
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new ()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)

    }
}

    pub fn get_configuration() ->Result<Settings, config::ConfigError>{

        // tell the software where to get the path
        let base_path =std::env::current_dir()
                    .expect("failed to determine the current directory");

        //get the path from configuration file
        let configuration_directory = base_path.join("configuration");

        // get the var variables
        let environment: Environment = std::env::var("APP_ENVIRONMENT")
                .unwrap_or_else(|_| "local".into())
                .try_into()
                .expect("failed to parse APP_ENVIRONMENT");

        let environment_filename = format!("{}.yaml", environment.as_str());

        let settings = config::Config::builder()
            .add_source(
                // config::File::new("configuration.yaml", config::FileFormat::Yaml)

                //add the base.yaml configuration
                config::File::from(configuration_directory.join("base.yaml"))
                
            )
            // add the special source, which means we need to combine the base.yaml with the environment.yaml together.
            .add_source(config::File::from(configuration_directory.join(&environment_filename)))
            .add_source(config::Environment::with_prefix("APP")
                        .prefix_separator("_")
                        .separator("__")
                    )
            .build()?;
            settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str(){
            // if we match the local, then we goet th local mode, otherwise we got the production mode.
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported enviroment. \
                use either 'local' or 'production' instead.", other)
            )
        }
    }
}