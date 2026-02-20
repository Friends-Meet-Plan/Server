use crate::controllers::{
    auth_controller as auth_routes, friendship_controller as friendship_routes,
    invitations_controller as invitation_routes, users_controller as users_routes,
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
        friendship_routes::reject_friend_request,
        invitation_routes::create_invitation,
        invitation_routes::get_incoming_invitations,
        invitation_routes::get_outgoing_invitations,
        invitation_routes::get_invitation_by_id,
        invitation_routes::accept_invitation,
        invitation_routes::decline_invitation,
        invitation_routes::cancel_invitation,
        invitation_routes::get_user_calendar,
        invitation_routes::get_my_busydays
    ),
    components(
        schemas(
            crate::controllers::models::AuthRequestBody,
            crate::controllers::models::LoginRequestBody,
            crate::controllers::models::LoginResponse,
            crate::controllers::models::FriendIdBody,
            crate::controllers::models::UserDTO,
            crate::controllers::models::user_response::UserResponse,
            crate::controllers::models::update_user_request_body::UpdateUserRequestBody,
            crate::controllers::models::CreateInvitationRequest,
            crate::controllers::models::CreatedInvitationResponse,
            crate::controllers::models::InvitationResponse,
            crate::controllers::models::InvitationDateResponse,
            crate::controllers::models::AcceptInvitationRequest,
            crate::controllers::models::BusydayResponse,
            crate::controllers::models::PendingInviteResponse,
            crate::controllers::models::CalendarResponse
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Users", description = "User profile endpoints"),
        (name = "Friends", description = "Friendship endpoints"),
        (name = "Invitations", description = "Invitations endpoints"),
        (name = "Calendar", description = "Calendar endpoints")
    )
)]
pub struct ApiDoc;
