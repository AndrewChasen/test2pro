use std::net::TcpListener;
use test2pro::configuration::get_configuration;
use test2pro::telemetry::{get_subscriber, init_subscriber};
// use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use test2pro::startup::run;
use tokio;
// use secrecy::ExposeSecret;
// use env_logger::Env;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // we can create the following code as a function
    // LogTracer::init().expect("Failed to set logger");

    // let env_filter = EnvFilter::try_from_default_env()
    //     .unwrap_or_else(|_| EnvFilter::new("info"));

    // let formatting_layer = BunyanFormattingLayer::new("
    //     test2pro".into(), std::io::stdout);

    
    // let subscriber = Registry::default().with(env_filter)
    //     .with(JsonStorageLayer).with(formatting_layer);

    // set_global_default(subscriber).expect("failed to set subscriber");  

    let subscriber = get_subscriber("test2pro".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);


    let configuration = get_configuration().expect("failed to get configuration");
    // let listener = TcpListener::bind("127.0.0.1:8080")?;
    
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
        // &configuration.database.connection_string().expose_secret())
        // .expect("Failed to connect to postgres database");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await?;
    Ok(())
}

