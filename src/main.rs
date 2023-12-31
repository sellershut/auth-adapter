mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use sea_orm::Database;
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("missing db url in env");
    let conn = Database::connect(&db_url).await?;
    let adapter = Arc::new(conn);

    let app = Router::new()
        .route("/health", get(routes::health))
        .route(
            "/users",
            post(routes::create_user)
                .get(routes::get_users)
                .delete(routes::delete_user)
                .put(routes::update_user),
        )
        .route(
            "/accounts",
            post(routes::create_account).delete(routes::delete_account),
        )
        .route(
            "/session",
            post(routes::create_session)
                .put(routes::update_session)
                .delete(routes::delete_session),
        )
        .route(
            "/verification-token",
            post(routes::create_verif_token).delete(routes::delete_verif_token),
        )
        .route("/session-user", get(routes::get_session_and_user))
        .with_state(adapter);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
