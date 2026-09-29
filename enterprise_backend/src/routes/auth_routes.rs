use crate::{
    domain::user::{AuthTokenResponse, CreateUserDto, LoginUserDto, UserResponse},
    error::AppError,
    extractors::auth::AuthenticatedUser,
    repository::user_repo::UserRepository,
    services::auth_service::AuthService,
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User successfully registered", body = AuthTokenResponse),
        (status = 400, description = "Validation or duplicate email error")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<CreateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    if dto.email.trim().is_empty()
        || dto.password.trim().is_empty()
        || dto.full_name.trim().is_empty()
    {
        return Err(AppError::BadRequest(
            "Email, password, and full_name are required".to_string(),
        ));
    }

    if let Some(_existing) = UserRepository::find_by_email(&state.db, &dto.email).await? {
        return Err(AppError::Conflict(
            "User with this email already exists".to_string(),
        ));
    }

    let user = UserRepository::create(&state.db, dto).await?;
    let token = AuthService::generate_jwt(
        user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    let response = AuthTokenResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginUserDto,
    responses(
        (status = 200, description = "Login successful", body = AuthTokenResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginUserDto>,
) -> Result<impl IntoResponse, AppError> {
    let user = UserRepository::find_by_email(&state.db, &dto.email)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

    let is_valid = AuthService::verify_password(&dto.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    let token = AuthService::generate_jwt(
        user.id,
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    let response = AuthTokenResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Get current authenticated user profile", body = UserResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let user = UserRepository::find_by_id(&state.db, auth_user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("User profile not found".to_string()))?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}
