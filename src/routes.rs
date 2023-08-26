use std::{collections::HashMap, sync::Arc};

use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Form, Json,
};
use entities::{
    account, session, session::Model as Session, user, user::Model as User, verification_token,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};

/// Find a user in the database. If no query is provided, all users are returned.
#[derive(Debug, Deserialize)]
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

#[debug_handler]
pub async fn create_user(
    State(state): State<Arc<DatabaseConnection>>,
    Json(payload): Json<user::Model>,
) -> Result<Json<user::Model>, StatusCode> {
    let item: user::ActiveModel = payload.into();
    match item.insert(&*state).await {
        Ok(value) => Ok(Json(value)),
        Err(e) => {
            eprintln!("{e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserResult {
    Single(user::Model),
    Multiple(Vec<user::Model>),
}

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

#[debug_handler]
pub async fn create_session(
    State(state): State<Arc<DatabaseConnection>>,
    Json(payload): Json<session::Model>,
) -> Result<Json<Session>, StatusCode> {
    let item: session::ActiveModel = payload.into();

    match item.insert(&*state).await {
        Ok(value) => Ok(Json(value)),
        Err(e) => {
            eprintln!("{e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserAndSession {
    pub user: user::Model,
    pub session: session::Model,
}

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

pub async fn update_session(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
    Form(form): Form<Session>,
) -> impl IntoResponse {
    println!("{query:#?}");
    if let Some(id) = query.get("id") {
        if let Ok(Some(session)) = session::Entity::find()
            .filter(session::Column::SessionToken.eq(id))
            .one(&*state)
            .await
        {
            let mut session: session::ActiveModel = session.into();
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

pub async fn delete_verif_token(
    State(state): State<Arc<DatabaseConnection>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<verification_token::Model>, StatusCode> {
    if let Some(id) = query.get("id") {
        if let Ok(Some(verif_token)) = verification_token::Entity::find()
            .filter(verification_token::Column::Identifier.eq(id))
            .one(&*state)
            .await
        {
            let return_value = verif_token.clone();
            if let Err(e) = verif_token.delete(&*state).await {
                eprintln!("{e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(Json(return_value))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        eprintln!("No parameters provided");
        Err(StatusCode::UNPROCESSABLE_ENTITY)
    }
}
