use axum::{
    Form, Json, extract::Extension, 
    http::{StatusCode, header::SET_COOKIE}, 
    response::{Html, IntoResponse, Redirect, Response}
};
use serde::{Serialize, Deserialize};
use crate::database::ChatDatabase;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub username: String,
}

pub async fn auth_get() -> Html<&'static str> {
    Html(include_str!("../templates/authentication.html"))
}

pub async fn login_handler(
    Extension(db): Extension<ChatDatabase>,
    Form(form): Form<LoginRequest>,
) -> Response {
    let username = form.username.trim();
    let password = form.password.trim();
    let password_hash = hash_password(password);

    if username.is_empty() {
        return (StatusCode::BAD_REQUEST, "Username cannot be empty").into_response();
    }

    if username.len() > 20 {
        return (StatusCode::BAD_REQUEST, "Username too long").into_response();
    }

    if password.is_empty() {
        return (StatusCode::BAD_REQUEST, "Password cannot be empty").into_response();
    }

    if password.len() < 6 {
        return (StatusCode::BAD_REQUEST, "Password too short (min 6 chars)").into_response();
    }

    match db.get_user_by_username(username) {
        Ok(Some(_user)) => {
            // User exists - verify password
            match db.verify_password(username, &password_hash) {
                Ok(true) => {
                    let session_id = format!("{}-{}", username, uuid::Uuid::new_v4());
                    match db.update_session(username, &session_id) {
                        Ok(_) => {
                            println!("User '{}' logged in!", username);
                            let cookie = format!(
                                "session_id={}; Path=/; Max-Age=604800; SameSite=Strict",
                                session_id
                            );
                            
                            (
                                [(SET_COOKIE, cookie)],
                                Redirect::to("/chat")
                            ).into_response()
                        },
                        Err(e) => {
                            eprintln!("Session update failed: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update session").into_response()
                        }
                    }
                },
                Ok(false) => {
                    (StatusCode::UNAUTHORIZED, "Invalid password").into_response()
                },
                Err(e) => {
                    eprintln!("Password verification error: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
                }
            }
        },
        Ok(None) => {
            // User doesn't exist - register new user
            let session_id = format!("{}-{}", username, uuid::Uuid::new_v4());
            
            match db.register_user(username, &password_hash, &session_id) {
                Ok(_) => {
                    println!("User '{}' registered", username);
                    let cookie = format!(
                        "session_id={}; Path=/; Max-Age=604800; SameSite=Strict",
                        session_id
                    );
                    
                    (
                        [(SET_COOKIE, cookie)],
                        Redirect::to("/chat")
                    ).into_response()
                },
                Err(e) => {
                    eprintln!("Registration failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to register user").into_response()
                }
            }
        },
        Err(e) => {
            eprintln!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

pub async fn get_user_handler(
    Extension(db): Extension<ChatDatabase>,
    headers: axum::http::HeaderMap,
) -> Response {
    // Extract session_id from cookie
    let session_id = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
                .find(|c| c.trim().starts_with("session_id="))
                .map(|c| c.trim().strip_prefix("session_id=").unwrap())
        });

    match session_id {
        Some(sid) => {
            match db.get_user_by_session(sid) {
                Ok(Some(user)) => {
                    // Return username as JSON
                    Json(UserResponse {
                        username: user.username,
                    }).into_response()
                },
                Ok(None) => {
                    (StatusCode::UNAUTHORIZED, "Invalid session").into_response()
                },
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
                }
            }
        },
        None => {
            (StatusCode::UNAUTHORIZED, "No session found").into_response()
        }
    }
}

fn hash_password(password: &str) -> String {
    format!("{:x}", Sha256::digest(password.as_bytes()))
}