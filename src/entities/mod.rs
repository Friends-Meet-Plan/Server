pub mod user;
pub mod friendship;
pub use user::ActiveModel as UserActiveModel;
pub use user::Column as UserColumn;
pub use user::Entity as User;
pub use friendship::ActiveModel as FriendshipActiveModel;
pub use friendship::Column as FriendshipColumn;
pub use friendship::Entity as Friendship;
