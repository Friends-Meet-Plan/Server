use crate::controllers::{
    auth_controller as auth_routes,
    friendship_controller as friendship_routes,
    users_controller as users_routes
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_routes::register,
        auth_routes::login,
        users_routes::get_me,
        users_routes::update_me,
        users_routes::get_user_by_id,
        users_routes::search_users,
        friendship_routes::get_friends,
        friendship_routes::friend_request,
        friendship_routes::get_incoming,
        friendship_routes::get_outgoing,
        friendship_routes::accept_friend_request,
        friendship_routes::remove_friend,
        friendship_routes::reject_friend_request
    ),
    components(
        schemas(
            crate::controllers::models::AuthRequestBody,
            crate::controllers::models::LoginRequestBody,
            crate::controllers::models::LoginResponse,
            crate::controllers::models::FriendIdBody,
            crate::controllers::models::UserDTO,
            crate::controllers::models::user_response::UserResponse,
            crate::controllers::models::update_user_request_body::UpdateUserRequestBody
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Users", description = "User profile endpoints"),
        (name = "Friends", description = "Friendship endpoints")
    )
)]
pub struct ApiDoc;
