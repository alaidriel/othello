use crate::server::{
    entities::{
        friend::Column as FriendColumn,
        friend_request::Column as FriendRequestColumn,
        game::{Column as GameColumn, Model},
        member::Column,
        prelude::{Friend, FriendRequest, Game},
    },
    extractors::User,
    handlers::StringError,
    helpers,
    state::AppState,
    strings, validate_password, validate_username,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, Value,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    current: String,
    new: String,
    confirmed: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMeRequest {
    username: Option<String>,
    password: Option<UpdatePasswordRequest>,
}

/// Fetch the current user's information.
pub async fn me(user: User) -> Result<impl IntoResponse, Response> {
    Ok(user)
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    user: User,
    Json(body): Json<UpdateMeRequest>,
) -> Result<impl IntoResponse, Response> {
    let stored = helpers::get_user(&state, &user.id.to_string(), false).await?;
    match body {
        UpdateMeRequest {
            username: Some(username),
            password: None,
        } => {
            validate_username(username.as_str())?;
            // Check if the username is already taken.
            if helpers::get_user(&state, &username, true).await.is_ok() {
                return Err(
                    StringError(strings::USERNAME_TAKEN.into(), StatusCode::CONFLICT)
                        .into_response(),
                );
            }
            let mut active = stored.into_active_model();
            active.set(
                Column::Username,
                Value::String(Some(Box::new(username.to_string()))),
            );
            active
                .save(state.database.as_ref())
                .await
                .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
            Ok(super::Response::new(json!({}), StatusCode::OK))
        }
        UpdateMeRequest {
            username: None,
            password:
                Some(UpdatePasswordRequest {
                    current,
                    new,
                    confirmed,
                }),
        } => {
            if new != confirmed {
                return Err(StringError(
                    strings::PASSWORD_MISMATCH.into(),
                    StatusCode::BAD_REQUEST,
                )
                .into_response());
            }
            helpers::ensure_valid_password(&stored.password, &current)?;
            validate_password(confirmed.as_str())?;
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let hashed = argon2
                .hash_password(new.as_bytes(), &salt)
                .map_err(|_| {
                    StringError(
                        strings::INVALID_PASSWORD_FORMAT.to_string(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })
                .map(|hashed| hashed.to_string())?;
            let mut active = stored.into_active_model();
            active.set(
                Column::Password,
                Value::String(Some(Box::new(hashed.to_string()))),
            );
            active
                .save(state.database.as_ref())
                .await
                .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
            Ok(super::Response::new(json!({}), StatusCode::OK))
        }
        _ => Err(StringError(strings::BAD_REQUEST.into(), StatusCode::BAD_REQUEST).into_response()),
    }
}

/// Fetch the games the current user is participating in.
pub async fn active_games(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let games = Game::find()
        .filter(
            GameColumn::Host
                .eq(user.id.to_string())
                .or(GameColumn::Guest.eq(user.id.to_string()))
                // Include only games that are active here, not pending games.
                // Pending games are on a separate endpoint.
                .and(GameColumn::Pending.eq(false)),
        )
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let resp = create_games_resp(state, &user, games).await?;
    Ok(super::Response::new(resp, StatusCode::OK))
}

/// Fetch the games the current user is currently awaiting a response for.
pub async fn pending_games(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let games = Game::find()
        .filter(
            GameColumn::Host
                .eq(user.id.to_string())
                .or(GameColumn::Guest.eq(user.id.to_string()))
                .and(GameColumn::Pending.eq(true)),
        )
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let resp = create_games_resp(state, &user, games).await?;
    Ok(super::Response::new(resp, StatusCode::OK))
}

/// Fetch the friend requests the current user has received.
pub async fn incoming(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let frs = FriendRequest::find()
        .filter(FriendRequestColumn::Recipient.eq(user.id))
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut incoming = vec![];
    for fr in &frs {
        let sender = helpers::get_user(&state, &fr.sender.to_string(), false).await?;
        incoming.push(json!({
            "sender": sender.username,
        }));
    }
    Ok(super::Response::new(incoming, StatusCode::OK))
}

/// Fetch the friend requests the current user has sent.
pub async fn outgoing(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let frs = FriendRequest::find()
        .filter(FriendRequestColumn::Sender.eq(user.id))
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut outgoing = vec![];
    for fr in &frs {
        let recipient = helpers::get_user(&state, &fr.recipient.to_string(), false).await?;
        outgoing.push(json!({
            "recipient": recipient.username,
        }));
    }
    Ok(super::Response::new(outgoing, StatusCode::OK))
}

/// Fetch the friends of the current user.
pub async fn friends(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let friends = Friend::find()
        .filter(FriendColumn::A.eq(user.id).or(FriendColumn::B.eq(user.id)))
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut f = vec![];
    for friend in &friends {
        let id = if friend.a == user.id {
            &friend.b
        } else {
            &friend.a
        };
        let friend = helpers::get_user(&state, &id.to_string(), false).await?;
        f.push(json!({
            "username": friend.username,
        }));
    }
    Ok(super::Response::new(f, StatusCode::OK))
}

/// Remove a friend from the current user's friend list.
pub async fn remove_friend(
    State(state): State<Arc<AppState>>,
    user: User,
    Path(friend): Path<String>,
) -> Result<impl IntoResponse, Response> {
    let friend = helpers::get_user(&state, &friend, true).await?;
    let Some(friend) = Friend::find()
        .filter(
            FriendColumn::A
                .eq(user.id)
                .and(FriendColumn::B.eq(friend.id))
                .or(FriendColumn::A
                    .eq(friend.id)
                    .and(FriendColumn::B.eq(user.id))),
        )
        .one(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
    else {
        return Err(
            StringError(strings::FRIEND_NOT_FOUND.into(), StatusCode::NOT_FOUND).into_response(),
        );
    };
    let result = friend
        .delete(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(super::Response::new(
        json!({ "affected": result.rows_affected }),
        StatusCode::OK,
    ))
}

async fn create_games_resp(
    state: Arc<AppState>,
    user: &User,
    games: Vec<Model>,
) -> Result<Vec<serde_json::Value>, Response> {
    let mut resp = vec![];
    for g in &games {
        let id = if user.id.to_string() == g.host {
            &g.guest
        } else {
            &g.host
        };
        let host = helpers::get_user(&state, g.host.as_str(), false).await?;
        let opponent = helpers::get_user(&state, id, false).await?;
        resp.push(json!({
            "id": g.id,
            "host": host.username,
            "opponent": opponent.username,
            "ended": g.ended,
        }));
    }
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::server::{self, handlers::Response};
    use test_utils::{function, Client, Map};

    #[tokio::test]
    async fn me() {
        let database = sea_orm::Database::connect(server::TEST_DATABASE_URI)
            .await
            .unwrap();
        let redis = redis::Client::open(server::TEST_REDIS_URI).unwrap();
        let state = Arc::new(server::AppState::new(database, redis));
        let url = test_utils::init(crate::server::app(state)).await;
        let client = Client::authenticated(&[&function!()], &url, true).await;
        let resp: Response<Map> = client.get(&url, "/@me").await;
        assert_eq!(resp.message["username"], function!());
    }
}
