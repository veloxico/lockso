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
use lockso_core::models::group::{
    AddMemberRequest, CreateGroupRequest, GroupMember, UpdateGroupRequest,
    UserGroupListItem, UserGroupView, UserGroup,
};
use lockso_core::models::activity_log::ActivityAction;
use lockso_core::services::{activity_log_service, group_service, user_management_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route(
            "/{id}",
            get(get_group)
                .put(update_group)
                .delete(delete_group),
        )
        .route(
            "/{id}/members",
            get(list_members).post(add_member),
        )
        .route("/{id}/members/{user_id}", axum::routing::delete(remove_member))
}

/// GET /v1/groups
async fn list_groups(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> Result<Json<Vec<UserGroupListItem>>, AppError> {
    let groups = group_service::list_groups(&state.db).await?;
    Ok(Json(groups))
}

/// GET /v1/groups/:id
async fn get_group(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserGroupView>, AppError> {
    let group = group_service::get_group(&state.db, id).await?;
    Ok(Json(group))
}

/// POST /v1/groups
async fn create_group(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<CreateGroupRequest>,
) -> Result<Json<UserGroup>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let group = group_service::create_group(
        &state.db,
        auth.user_id,
        &input.name,
        &input.description,
    )
    .await?;

    activity_log_service::log_activity(
        &state.db,
        Some(auth.user_id),
        ActivityAction::GROUP_CREATED,
        Some("group"),
        Some(group.id),
        None,
        auth.session.client_ip.as_deref(),
        auth.session.user_agent.as_deref(),
        serde_json::json!({"name": group.name}),
    )
    .await;

    Ok(Json(group))
}

/// PUT /v1/groups/:id
async fn update_group(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateGroupRequest>,
) -> Result<Json<UserGroup>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let group = group_service::update_group(
        &state.db,
        id,
        input.name.as_deref(),
        input.description.as_deref(),
    )
    .await?;

    activity_log_service::log_activity(
        &state.db,
        Some(auth.user_id),
        ActivityAction::GROUP_UPDATED,
        Some("group"),
        Some(id),
        None,
        auth.session.client_ip.as_deref(),
        auth.session.user_agent.as_deref(),
        serde_json::json!({"name": group.name}),
    )
    .await;

    Ok(Json(group))
}

/// DELETE /v1/groups/:id
async fn delete_group(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;
    group_service::delete_group(&state.db, id).await?;

    activity_log_service::log_activity(
        &state.db,
        Some(auth.user_id),
        ActivityAction::GROUP_DELETED,
        Some("group"),
        Some(id),
        None,
        auth.session.client_ip.as_deref(),
        auth.session.user_agent.as_deref(),
        serde_json::json!({}),
    )
    .await;

    Ok(())
}

/// GET /v1/groups/:id/members
async fn list_members(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupMember>>, AppError> {
    let group = group_service::get_group(&state.db, id).await?;
    Ok(Json(group.members))
}

/// POST /v1/groups/:id/members
async fn add_member(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<AddMemberRequest>,
) -> Result<Json<GroupMember>, AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;

    let member = group_service::add_member(&state.db, id, input.user_id, auth.user_id).await?;

    activity_log_service::log_activity(
        &state.db,
        Some(auth.user_id),
        ActivityAction::GROUP_MEMBER_ADDED,
        Some("group"),
        Some(id),
        None,
        auth.session.client_ip.as_deref(),
        auth.session.user_agent.as_deref(),
        serde_json::json!({"userId": input.user_id.to_string()}),
    )
    .await;

    Ok(Json(member))
}

/// DELETE /v1/groups/:id/members/:user_id
async fn remove_member(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    validate_csrf(&state, &auth, &headers).await?;
    user_management_service::require_admin(&state.db, auth.user_id).await?;
    group_service::remove_member(&state.db, id, user_id).await?;

    activity_log_service::log_activity(
        &state.db,
        Some(auth.user_id),
        ActivityAction::GROUP_MEMBER_REMOVED,
        Some("group"),
        Some(id),
        None,
        auth.session.client_ip.as_deref(),
        auth.session.user_agent.as_deref(),
        serde_json::json!({"userId": user_id.to_string()}),
    )
    .await;

    Ok(())
}
