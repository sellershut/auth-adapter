use std::{collections::HashMap, sync::Arc};

use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Form, Json,
};
use entities::{
    account, session,
    session::Model as Session,
    user,
    user::Model as User,
    utoipa::{self, IntoParams, ToSchema},
    verification_token,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
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
pub enum UserResult {
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
) -> Result<Json<UserResult>, StatusCode> {
    if let Some(id) = params.id {
        match user::Entity::find_by_id(id).one(&*state).await {
            Ok(users) => {
                if let Some(user) = users {
                    return Ok(Json(UserResult::Single(user)));
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
                    return Ok(Json(UserResult::Single(user)));
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
                    return Ok(Json(UserResult::Multiple(users.to_owned())));
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
                    return Ok(Json(UserResult::Multiple(users.to_owned())));
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
                    return Ok(Json(UserResult::Multiple(users.to_owned())));
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
        Ok(users) => Ok(Json(UserResult::Multiple(users))),
        Err(e) => {
            eprintln!("{e}");
            Err(StatusCode::NO_CONTENT)
        }
    }
}

/// Update a user by given id.
#[utoipa::path(
    put,
    path = "/users",
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 404, description = "User not found"),
        (status = 422, description = "User id not provided"),
        (status = 500, description = "Could not update user")
    ),
    params(
        ("id" = String, Query, description = "User Id")
    ),
    request_body(content = Model, content_type = "application/x-www-form-urlencoded")
)]
pub async fn update_user(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
    Form(form): Form<User>,
) -> impl IntoResponse {
    println!("{query:#?}");
    if let Some(id) = query.get("id") {
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
            if let Err(e) = user.update(&*state).await {
                eprintln!("{e}");
                StatusCode::INTERNAL_SERVER_ERROR
            } else {
                StatusCode::OK
            }
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        eprintln!("No parameters provided");
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

/// Delete a user by given id.
#[utoipa::path(
    delete,
    path = "/users",
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 404, description = "User not found"),
        (status = 422, description = "User id not provided"),
        (status = 500, description = "Could not delete user")
    ),
    params(
        ("id" = String, Query, description = "User Id")
    ),
)]
pub async fn delete_user(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(id) = query.get("id") {
        if let Ok(Some(user)) = user::Entity::find_by_id(id).one(&*state).await {
            if let Err(e) = user.delete(&*state).await {
                eprintln!("{e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        eprintln!("No parameters provided");
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

pub async fn health() -> &'static str {
    "hello"
}

/// Create new Account
#[utoipa::path(
        post,
        path = "/accounts",
        request_body = Model,
        responses(
            (status = 201, description = "Account created successfully", body = Model),
        )
)]
#[debug_handler]
pub async fn create_account(
    State(state): State<Arc<DatabaseConnection>>,
    Json(payload): Json<account::Model>,
) -> impl IntoResponse {
    let item: account::ActiveModel = payload.into();
    if let Err(e) = item.insert(&*state).await {
        eprintln!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::CREATED
    }
}

/// Delete an account
#[utoipa::path(
    delete,
    path = "/accounts",
    responses(
        (status = 200, description = "Account deleted successfully"),
        (status = 404, description = "Account not found"),
        (status = 422, description = "Account provider id not provided"),
        (status = 500, description = "Could not delete user")
    ),
    params(
        ("name" = String, Query, description = "Provider name"),
        ("id" = String, Query, description = "provider Account Id")
    ),
)]
pub async fn delete_account(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(Some((id, name))) = query
        .get("id")
        .map(|id| query.get("name").map(|name| (id, name)))
    {
        if let Ok(Some(account)) = account::Entity::find()
            .filter(account::Column::ProviderAccountId.eq(id))
            .filter(account::Column::Provider.eq(name))
            .one(&*state)
            .await
        {
            if let Err(e) = account.delete(&*state).await {
                eprintln!("{e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        eprintln!("No parameters provided");
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

/// Create new Session
#[utoipa::path(
        post,
        path = "/session",
        request_body = Model,
        responses(
            (status = 201, description = "Session created successfully", body = Model),
        )
)]
#[debug_handler]
pub async fn create_session(
    State(state): State<Arc<DatabaseConnection>>,
    Json(payload): Json<session::Model>,
) -> impl IntoResponse {
    let item: session::ActiveModel = payload.into();
    if let Err(e) = item.insert(&*state).await {
        eprintln!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::CREATED
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserAndSession {
    pub user: user::Model,
    pub session: session::Model,
}

/// Get user and session by given session token.
#[utoipa::path(
    get,
    path = "/session-user",
    responses(
        (status = 200, description = "Session and user found"),
        (status = 422, description = "SessionToken not provided"),
        (status = 500, description = "Could not get session and user")
    ),
    params(
        ("sessionToken" = String, Query, description = "Session Token")
    ),
)]
#[debug_handler]
pub async fn get_session_and_user(
    Query(query): Query<HashMap<String, String>>,
    State(state): State<Arc<DatabaseConnection>>,
) -> Result<Json<UserAndSession>, StatusCode> {
    if let Some(id) = query.get("sessionToken") {
        if let Ok(Some(session)) = session::Entity::find()
            .filter(session::Column::SessionToken.eq(id))
            .one(&*state)
            .await
        {
            if let Ok(Some(user)) = user::Entity::find_by_id(&session.user_id)
                .one(&*state)
                .await
            {
                Ok(Json(UserAndSession { user, session }))
            } else {
                Err(StatusCode::NO_CONTENT)
            }
        } else {
            Err(StatusCode::NO_CONTENT)
        }
    } else {
        eprintln!("No parameters provided");
        Err(StatusCode::UNPROCESSABLE_ENTITY)
    }
}

/// Update a user by given id.
#[utoipa::path(
    put,
    path = "/session",
    responses(
        (status = 200, description = "Session updated successfully"),
        (status = 404, description = "Session not found"),
        (status = 422, description = "Session Token not provided"),
        (status = 500, description = "Could not update session")
    ),
    params(
        ("sessionToken" = String, Query, description = "Session Token")
    ),
    request_body(content = Model, content_type = "application/x-www-form-urlencoded")
)]
pub async fn update_session(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
    Form(form): Form<Session>,
) -> impl IntoResponse {
    println!("{query:#?}");
    if let Some(id) = query.get("id") {
        if let Ok(Some(session)) = session::Entity::find_by_id(id).one(&*state).await {
            let mut session: session::ActiveModel = session.into();
            session.id = Set(form.id);
            session.user_id = Set(form.user_id);
            session.expires = Set(form.expires);
            session.session_token = Set(form.session_token);
            if let Err(e) = session.update(&*state).await {
                eprintln!("{e}");
                StatusCode::INTERNAL_SERVER_ERROR
            } else {
                StatusCode::OK
            }
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        eprintln!("No parameters provided");
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

/// Delete a session by given token.
#[utoipa::path(
    delete,
    path = "/session",
    responses(
        (status = 200, description = "Session deleted successfully"),
        (status = 404, description = "Session not found"),
        (status = 422, description = "Session id not provided"),
        (status = 500, description = "Could not delete session")
    ),
    params(
        ("sessionToken" = String, Query, description = "Session Token")
    ),
)]
pub async fn delete_session(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(id) = query.get("sessionToken") {
        if let Ok(Some(session)) = session::Entity::find()
            .filter(session::Column::SessionToken.eq(id))
            .one(&*state)
            .await
        {
            if let Err(e) = session.delete(&*state).await {
                eprintln!("{e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        eprintln!("No parameters provided");
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

/// Create new Session
#[utoipa::path(
        post,
        path = "/verification-token",
        request_body = Model,
        responses(
            (status = 201, description = "Verification Token created successfully", body = Model),
        )
)]
#[debug_handler]
pub async fn create_verif_token(
    State(state): State<Arc<DatabaseConnection>>,
    Json(payload): Json<verification_token::Model>,
) -> impl IntoResponse {
    let item: verification_token::ActiveModel = payload.into();
    if let Err(e) = item.insert(&*state).await {
        eprintln!("{e}");
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::CREATED
    }
}

/// Delete a verification token by given id.
#[utoipa::path(
    delete,
    path = "/verification-token",
    responses(
        (status = 200, description = "Verification Token deleted successfully"),
        (status = 404, description = "Verification Token not found"),
        (status = 422, description = "Verification Token id not provided"),
        (status = 500, description = "Could not delete verification token")
    ),
    params(
        ("id" = String, Query, description = "Verification Token Id")
    ),
)]
pub async fn delete_verif_token(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(id) = query.get("id") {
        if let Ok(Some(verif_token)) = verification_token::Entity::find()
            .filter(verification_token::Column::Identifier.eq(id))
            .one(&*state)
            .await
        {
            if let Err(e) = verif_token.delete(&*state).await {
                eprintln!("{e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        eprintln!("No parameters provided");
        StatusCode::UNPROCESSABLE_ENTITY
    }
}
