use crate::controllers::{
    auth_controller as auth_routes, calendar_controller as calendar_routes,
    event_controller as event_routes, friendship_controller as friendship_routes,
    users_controller as users_routes,
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
        calendar_routes::get_my_calendar,
        calendar_routes::get_user_calendar,
        event_routes::create_event,
        event_routes::get_event,
        event_routes::get_events,
        event_routes::update_event,
        event_routes::cancel_event,
        event_routes::get_event_participants,
        event_routes::accept_event,
        event_routes::decline_event
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
            crate::controllers::models::calendar::BusydayResponse,
            crate::controllers::models::calendar::PendingInviteResponse,
            crate::controllers::models::calendar::CalendarResponse,
            crate::controllers::models::events::CreateEventBody,
            crate::controllers::models::events::UpdateEventBody,
            crate::controllers::models::events::EventScope,
            crate::controllers::models::events::EventResponse,
            crate::controllers::models::events::ParticipantResponse
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Users", description = "User profile endpoints"),
        (name = "Friends", description = "Friendship endpoints"),
        (name = "Calendar", description = "Calendar endpoints"),
        (name = "Events", description = "Events endpoints")
    )
)]
pub struct ApiDoc;
