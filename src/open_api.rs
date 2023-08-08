use crate::routes::{UserData, __path_create_user, __path_get_users, __path_update_user};
use entities::utoipa::OpenApi;
use entities::{user::Model as User, utoipa};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        get_users,
        update_user
    ),
    components(
        schemas(User, UserData),
    ),
    tags(
        (name = "Sample Project", description = "Auth Adapter")
    )
)]
pub struct ApiDoc;
