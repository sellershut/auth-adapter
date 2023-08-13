use crate::routes::{
    __path_create_account, __path_create_session, __path_create_user, __path_delete_account,
    __path_delete_user, __path_get_session_and_user, __path_get_users, __path_update_session,
    __path_update_user,
};
use entities::utoipa::OpenApi;
use entities::{account::Model as Account, session::Model as Session, user::Model as User, utoipa};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        get_users,
        update_user,
        delete_user,
        create_account,
        delete_account,
        create_session,
        get_session_and_user,
        update_session
    ),
    components(
        schemas(User, Account, Session),
    ),
    tags(
        (name = "Sample Project", description = "Auth Adapter")
    )
)]
pub struct ApiDoc;
