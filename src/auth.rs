use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
    extract::Extension,
    Form,
};
use serde::{Serialize,Deserialize};
use crate::database::ChatDatabase;
use sha2::{Sha256, Digest};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub session_id: Option<String>,
}

pub async fn login_handler(
    Extension(db): Extension<ChatDatabase>,
    Form(form): Form<LoginRequest>,
) -> Response {
    let username = form.username.as_str().trim();
    let password = form.password.as_str().trim();
    let hash = format!("{:x}", Sha256::digest(password.as_bytes()));

    if username.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(LoginResponse {
                success: false,
                message: "Username cannot be empty".to_string(),
                session_id: None,
            })
        ).into_response();
    }

    if username.len() > 20 {
        return (
            StatusCode::BAD_REQUEST,
            Json(LoginResponse {
                success: false,
                message: "Username too long".to_string(),
                session_id: None,
            })
        ).into_response();
    }

    match db.username_exists(username) {
        Ok(true) => {
            (StatusCode::CONFLICT,
            Json(LoginResponse {
                success: false,
                message: "Username is already taken".to_string(),
                session_id: None,
            })
            ).into_response()
        },
        Ok(false) => {
            let session_id = format!("{}-{}",username, uuid::Uuid::new_v4());

            match db.register_user(username, &session_id) {
                Ok(_) => {
                    Json(LoginResponse {
                        success: true,
                        message: "Login successful".to_string(),
                        session_id: Some(session_id),
                    }).into_response()
                },
                Err(e) => {
                    eprintln!("Registration Failed: {}",e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginResponse{
                            success: false,
                            message: "Failed to register user".to_string(),
                            session_id: None,
                        })
                    ).into_response()
                }
            }
        },
        Err(e) => {
            eprintln!("Database error checking username: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    success: false,
                    message: "Database error".to_string(),
                    session_id: None,
                })
            ).into_response()
        }
    }
}