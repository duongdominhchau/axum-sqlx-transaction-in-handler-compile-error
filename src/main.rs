use axum::{Router, extract::State, routing::get};
use sqlx::{Acquire, PgPool, Postgres, postgres::PgPoolOptions};
use tokio::net::TcpListener;

async fn foo(conn: impl Acquire<'_, Database = Postgres>) {
    let mut conn = conn.acquire().await.unwrap();
    sqlx::query("select 1 + $1")
        .bind(1)
        .execute(&mut *conn)
        .await
        .unwrap();
}

async fn handler_with_error(State(pool): State<PgPool>) {
    let mut tx = pool.begin().await.unwrap();
    foo(&mut *tx).await;
    tx.commit().await.unwrap();
}

async fn bar(conn: impl Acquire<'_, Database = Postgres>) {
    let mut tx = conn.begin().await.unwrap();
    sqlx::query("select 1 + $1")
        .bind(1)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}
async fn handler_without_error(State(pool): State<PgPool>) {
    foo(&pool).await;
}

async fn handler_without_error_but_incorrect(State(pool): State<PgPool>) {
    foo(&pool).await;
}

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new().connect("aaaaa").await.unwrap();
    let app = Router::new()
        .route("/error", get(handler_with_error))
        .route("/no-error", get(handler_without_error))
        .route("/no-error", get(handler_without_error_but_incorrect))
        .with_state(pool);
    let server = TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(server, app).await.unwrap();
}
