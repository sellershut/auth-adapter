use crate::routes::__path_create_user;
use entities::utoipa::OpenApi;
use entities::{user, utoipa};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user
    ),
    components(
        schemas(user::Model)
    ),
    tags(
        (name = "Sample Project", description = "Auth Adapter")
        )
    )]
pub struct ApiDoc;
