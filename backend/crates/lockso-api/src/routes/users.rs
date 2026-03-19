use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
};
use uuid::Uuid;

use crate::extractors::auth::AuthUser;
use crate::helpers::csrf::validate_csrf;
use crate::state::AppState;
use lockso_core::error::AppError;
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::models::user::UserView;
use lockso_core::models::user_role::UserRoleView;
use lockso_core::services::{activity_log_service, user_management_service};
use lockso_core::services::user_management_service::{
    AdminCreateUser, AdminUserListItem, SetUserBlocked, UpdateUserRole,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/roles", get(list_roles))
        .route("/{id}/role", axum::routing::put(update_role))
        .route("/{id}/block", axum::routing::put(set_blocked))
        .route("/{id}", axum::routing::delete(delete_user))
}

/// POST /v1/users
///
/// Create a new user (admin only).
async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<AdminCreateUser>,
) -> Result<Json<UserView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let user = user_management_service::admin_create_user(&state.db, input).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::REGISTER,
        Some("user"), Some(user.id), None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"createdByAdmin": true}),
    ).await;
    Ok(Json(user))
}

/// GET /v1/users
///
/// List all users with role info. Requires admin/owner.
async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<AdminUserListItem>>, AppError> {
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let users = user_management_service::list_users(&state.db).await?;
    Ok(Json(users))
}

/// GET /v1/users/roles
///
/// List all available roles.
async fn list_roles(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<UserRoleView>>, AppError> {
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let roles = user_management_service::list_roles(&state.db).await?;
    Ok(Json(roles))
}

/// PUT /v1/users/:id/role
///
/// Update a user's role. Requires admin/owner.
async fn update_role(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUserRole>,
) -> Result<Json<UserView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let user =
        user_management_service::update_user_role(&state.db, auth.user_id, id, input.role_id)
            .await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::USER_ROLE_CHANGED,
        Some("user"), Some(id), None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({"roleId": input.role_id.to_string()}),
    ).await;
    Ok(Json(user))
}

/// PUT /v1/users/:id/block
///
/// Block or unblock a user. Requires admin/owner.
async fn set_blocked(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<SetUserBlocked>,
) -> Result<Json<UserView>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let user =
        user_management_service::set_user_blocked(&state.db, auth.user_id, id, input.is_blocked)
            .await?;
    let action = if input.is_blocked { ActivityAction::USER_BLOCKED } else { ActivityAction::USER_UNBLOCKED };
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), action,
        Some("user"), Some(id), None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(Json(user))
}

/// DELETE /v1/users/:id
///
/// Delete a user. Requires admin/owner.
async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    user_management_service::delete_user(&state.db, auth.user_id, id).await?;
    activity_log_service::log_activity(
        &state.db, Some(auth.user_id), ActivityAction::USER_DELETED,
        Some("user"), Some(id), None,
        auth.session.client_ip.as_deref(), auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    ).await;
    Ok(())
}
