// Public-facing error messages
pub const USERNAME_TOO_SHORT: &str = "Username must be at least 3 characters.";
pub const PASSWORD_TOO_SHORT: &str = "Password must be at least 8 characters.";
pub const PASSWORD_NO_ALPHA: &str = "Password must contain at least one alphabetic character.";
pub const PASSWORD_NO_NUMERIC: &str = "Password must contain at least one number.";

// -- internal --
pub const FRIEND_REQUEST_ALREADY_SENT: &str = "friend request already sent";
pub const IDENTIFY_TIMEOUT: &str = "connection timed out";
pub const INVALID_GAME_ID: &str = "no game exists with specified id";
pub const INVALID_GAME_ID_FORMAT: &str = "invalid game id format (expected uuid)";
pub const INVALID_USERNAME: &str = "no user exists with specified username";
pub const USERNAME_TAKEN: &str = "that username is already taken";
pub const INVALID_PASSWORD_FORMAT: &str = "password failed to hash correctly";
pub const INVALID_PASSWORD: &str = "password does not match";
pub const INVALID_TOKEN: &str = "invalid user token";
pub const SESSION_COOKIE_NAME: &str = "sid";
pub const FRIEND_REQUEST_NOT_FOUND: &str = "no friend request exists from that user";
pub const FRIEND_NOT_FOUND: &str = "authenticated user is not friends with that user";
