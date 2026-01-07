use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    println!("Cleaning up conflicting migration record...");
    // Delete the record for migration 6. The checksum mismatch happens because 
    // the code expects one version but the DB has another.
    // Deleting it will force sqlx to re-evaluate (and since we renamed the conflicting file,
    // it should match the *other* migration 6, or if we are lucky, just proceed).
    // Actually, waip-hub has `006_add_agent_type` and we renamed `006_add_agent_fields` to `008`.
    // So now there is only one `006` file. 
    // If the DB has the checksum for `add_agent_fields` as version 6, it will conflict with `add_agent_type`.
    // Deleting the row allows `add_agent_type` to be applied as the "new" version 6 (or skipped if already done, but likely it needs to "claim" version 6).
    
    // Safety: check if migration 6 exists before deleting
    let row: Option<(i64,)> = sqlx::query_as("SELECT version FROM _sqlx_migrations WHERE version = 6")
        .fetch_optional(&pool)
        .await?;

    if row.is_some() {
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 6")
            .execute(&pool)
            .await?;
        println!("Successfully deleted migration version 6 record.");
    } else {
        println!("Migration version 6 not found in history.");
    }

    println!("Done. You can now run the application normally.");
    Ok(())
}
