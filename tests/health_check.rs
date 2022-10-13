use std::net::TcpListener;
use sqlx::{PgPool, Connection, Executor, PgConnection};
use test2pro::startup::run;
use test2pro::configuration::{get_configuration, DatabaseSettings};
use test2pro::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;
use once_cell::sync::Lazy;
// use secrecy::ExposeSecret;


static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level ="info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok(){
        let subscriber = get_subscriber(subscriber_name, 
            default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
    }else{
        let subscriber = get_subscriber("test".into(), "debug".into()
    ,std::io::sink);
            init_subscriber(subscriber);
    };    
});


pub struct TestApp{
    pub address: String,
    pub db_pool: PgPool,
}
async fn spwan_app()-> TestApp{
    //All other incocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();    
    let address = format!("http://127.0.0.1:{}", port);

    // here we use the same database as test, but if we want to redo it, then fail and 
    //should rollback to the start. so we will give a random database for it.
    let mut configuration = get_configuration().expect("failed to get configuration");

    //we define a new uuid into database_name, and so we will not get the 
    //database name from the configuration file.
    //here we only use the databaase name that we define here.
    configuration.database.database_name = Uuid::new_v4().to_string();


    let connection_pool = configure_database(&configuration.database).await;     

    let server = run(listener, connection_pool.clone()).expect("failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}


async fn configure_database(config:&DatabaseSettings) ->PgPool{

    let mut connection = PgConnection::connect_with(
        &config.without_db()
    ).await.expect("Couldn't connect to database named postgres");

    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await.expect("Couldn't create database named postgres");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await.expect("failed to connect to postgres database");

    sqlx::migrate!("./migrations").run(&connection_pool)
        .await
        .expect("failed to migrate the database");
    
    connection_pool


}
#[tokio::test]
pub async fn health_check_works()  {
    let app = spwan_app().await;
    
    let client = reqwest::Client::new();

    let response = client.get(&format!("{}/health_check",&app.address))
        .send()
        .await
        .expect("faiiled to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

}

#[tokio::test]
pub async fn subscribe_returns_a_200_for_valid_from_data(){
    let app = spwan_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscription",&app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to post data in subscriptions.");

    assert_eq!(200, response.status().as_u16());

    // this means we also save the data into the database by sqlx
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved subscription.");
    
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

}

#[tokio::test]
pub async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spwan_app().await;
    let client = reqwest::Client::new();

    let body = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];
    
    for (invaild_body, error_message) in body{
        let response = client
                .post(format!("{}/subscription", &app.address))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(invaild_body)
                .send()
                .await
                .expect("Failed to post data in subscriptions.");
        
        assert_eq!(400, response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}.",
        error_message
    );
    }
}