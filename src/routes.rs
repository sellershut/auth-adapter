use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Form, Json,
};
use entities::{
    account, user,
    utoipa::{self, IntoParams, ToSchema},
};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

/// Find a user in the database. If no query is provided, all users are returned.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct UserSearchQuery {
    /// Search by user's `id`.
    id: Option<String>,
    /// Search by user `email` address.
    email: Option<String>,
    /// Search by provider account `name`.
    provider: Option<String>,
    /// Search by provider account `id`.
    provider_account_id: Option<String>,
}

/// Create new User
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

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum User {
    Single(user::Model),
    Multiple(Vec<user::Model>),
}

#[utoipa::path(
        get,
        path = "/users",
        responses(
            (status = 200, description = "OK"),
            (status = 204, description = "User not found")
        ),
        params(
            UserSearchQuery,
        ),
)]
#[debug_handler]
pub async fn get_users(
    State(state): State<Arc<DatabaseConnection>>,
    Query(params): Query<UserSearchQuery>,
) -> Result<Json<User>, StatusCode> {
    if let Some(id) = params.id {
        match user::Entity::find_by_id(id).one(&*state).await {
            Ok(users) => {
                if let Some(user) = users {
                    return Ok(Json(User::Single(user)));
                } else {
                    return Err(StatusCode::NO_CONTENT);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(StatusCode::NO_CONTENT);
            }
        }
    } else if let Some(email) = params.email {
        match user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&*state)
            .await
        {
            Ok(item) => {
                if let Some(user) = item {
                    return Ok(Json(User::Single(user)));
                } else {
                    return Err(StatusCode::NO_CONTENT);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(StatusCode::NO_CONTENT);
            }
        }
    } else if params.provider_account_id.is_some() && params.provider.is_some() {
        let id = params.provider_account_id.unwrap();
        let name = params.provider.unwrap();
        match account::Entity::find()
            .filter(
                Condition::all()
                    .add(account::Column::Id.eq(id))
                    .add(account::Column::Provider.eq(name)),
            )
            .find_with_related(user::Entity)
            .all(&*state)
            .await
        {
            Ok(result) => {
                if let Some((_, users)) = result.first() {
                    return Ok(Json(User::Multiple(users.to_owned())));
                } else {
                    return Err(StatusCode::NO_CONTENT);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else if let Some(id) = params.provider_account_id {
        match account::Entity::find()
            .filter(Condition::all().add(account::Column::Id.eq(id)))
            .find_with_related(user::Entity)
            .all(&*state)
            .await
        {
            Ok(result) => {
                if let Some((_, users)) = result.first() {
                    return Ok(Json(User::Multiple(users.to_owned())));
                } else {
                    return Err(StatusCode::NO_CONTENT);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else if let Some(name) = params.provider {
        match account::Entity::find()
            .filter(Condition::all().add(account::Column::Provider.eq(name)))
            .find_with_related(user::Entity)
            .all(&*state)
            .await
        {
            Ok(result) => {
                if let Some((_, users)) = result.first() {
                    return Ok(Json(User::Multiple(users.to_owned())));
                } else {
                    return Err(StatusCode::NO_CONTENT);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
    match user::Entity::find().all(&*state).await {
        Ok(users) => Ok(Json(User::Multiple(users))),
        Err(e) => {
            eprintln!("{e}");
            Err(StatusCode::NO_CONTENT)
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserData {
    pub name: Option<String>,
    pub email: Option<String>,
    #[schema(value_type = String, format = DateTime, nullable)]
    pub email_verified: Option<DateTimeWithTimeZone>,
    pub image: Option<String>,
}

/// Update a user by given id. Return only status 200 on success or 404 if `User` is not found.
#[utoipa::path(
    put,
    path = "/users",
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 404, description = "User not found")
    ),
    params(
        ("id" = String, Path, description = "User id")
    ),
)]
pub async fn update_user(
    State(state): State<Arc<DatabaseConnection>>,
    Query(id): Query<String>,
    Form(form): Form<UserData>,
) {
    if let Ok(Some(user)) = user::Entity::find_by_id(id).one(&*state).await {
        let mut user: user::ActiveModel = user.into();
        if let Some(name) = form.name {
            user.name = Set(Some(name));
        }
        if let Some(email) = form.email {
            user.email = Set(Some(email));
        }
        if let Some(email_verified) = form.email_verified {
            user.email_verified = Set(Some(email_verified));
        }
        if let Some(image) = form.image {
            user.image = Set(Some(image));
        }
    }
}

pub async fn health() -> &'static str {
    "hello"
}
