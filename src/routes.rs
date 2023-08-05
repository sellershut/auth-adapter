use std::sync::Arc;

use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use entities::{user, utoipa};
use sea_orm::{ActiveModelTrait, DatabaseConnection};

/// Create new User
///
/// Tries to create a new Todo item to in-memory storage or fails with 409 conflict if already exists.
#[utoipa::path(
        post,
        path = "/users",
        request_body = Model,
        responses(
            (status = 201, description = "User created successfully", body = Model),
        )
)]
#[debug_handler]
pub async fn create_user(
    State(state): State<Arc<DatabaseConnection>>,
    Json(payload): Json<user::Model>,
) -> impl IntoResponse {
    let item: user::ActiveModel = payload.into();
    if let Err(e) = item.insert(&*state).await {
        eprintln!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::CREATED
    }
}

pub async fn get_user(State(state): State<Arc<DatabaseConnection>>) -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
