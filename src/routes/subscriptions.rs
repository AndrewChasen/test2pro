use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
// use tracing::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData{
    email: String,
    name: String,

}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        // request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
        form: web::Form<FormData>, 
        pool: web::Data<PgPool>)
        -> HttpResponse
        {
            // let request_id = Uuid::new_v4();

            // let request_span = tracing::info_span!(
            //     "Adding a new subscriber.", %request_id, 
            //     subscriber_email = %form.email,
            //     subscriber_name = %form.name
            // );

            // let _request_span_guard = request_span.enter();


            // let query_span = tracing::info_span!(
            //     "Saving new subscriber details in the database"
            // );
            // tracing::info!("request_id {} - Adding '{}' '{}' as a new subscriber.", request_id, form.email, form.name);
            // tracing::info!("request_id {} - Saving new subscriber details in the database", request_id);
            match insert_subscriber(&pool, &form).await
            // sqlx::query!(
            //     r#"
            //     INSERT INTO subscriptions (id, email, name, subscribed_at) 
            //     VALUES ($1,$2,$3,$4)
            //     "#,
            //     Uuid::new_v4(),
            //     form.email,
            //     form.name,
            //     Utc::now()
            // )
            // .execute(pool.get_ref())
            // .instrument(query_span)
    {
        Ok(_) => 
            // tracing::info!("request_id {} - new subscriber details have been saved successfully", request_id);
            HttpResponse::Ok().finish(),
        Err(e) => 
            // tracing::error!("request_id {} - Faled to execute query {:?}", request_id, e);
            // tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    form: &FormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at) 
        VALUES ($1,$2,$3,$4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
} 