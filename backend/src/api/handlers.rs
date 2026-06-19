use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::db::Database;
use super::models::*;

// ========== Application State ==========

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

// ========== Upstream Resolver Handlers ==========

pub async fn list_resolvers(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::db::UpstreamResolver>>>, StatusCode> {
    debug!("GET /api/v1/resolvers");

    match state.db.get_all_resolvers().await {
        Ok(resolvers) => {
            info!("Retrieved {} resolvers", resolvers.len());
            Ok(Json(ApiResponse::success(resolvers)))
        }
        Err(e) => {
            error!("Failed to get resolvers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_resolver(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::db::UpstreamResolver>>, StatusCode> {
    debug!("GET /api/v1/resolvers/{}", id);

    match state.db.get_resolver_by_id(id).await {
        Ok(Some(resolver)) => Ok(Json(ApiResponse::success(resolver))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get resolver {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_resolver(
    State(state): State<AppState>,
    Json(payload): Json<CreateResolverRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CreateResponse>>), StatusCode> {
    debug!("POST /api/v1/resolvers - {:?}", payload);

    match state
        .db
        .create_resolver(&payload.ip_address, &payload.domain)
        .await
    {
        Ok(id) => {
            info!("Created resolver {} - {} -> {}", id, payload.domain, payload.ip_address);
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse::success(CreateResponse { id })),
            ))
        }
        Err(e) => {
            error!("Failed to create resolver: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_resolver(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateResolverRequest>,
) -> Result<Json<ApiResponse<UpdateResponse>>, StatusCode> {
    debug!("PUT /api/v1/resolvers/{} - {:?}", id, payload);

    match state
        .db
        .update_resolver(
            id,
            payload.ip_address.as_deref(),
            payload.domain.as_deref(),
        )
        .await
    {
        Ok(true) => {
            info!("Updated resolver {}", id);
            Ok(Json(ApiResponse::success(UpdateResponse { updated: true })))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update resolver {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_resolver(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<DeleteResponse>>, StatusCode> {
    debug!("DELETE /api/v1/resolvers/{}", id);

    match state.db.delete_resolver(id).await {
        Ok(true) => {
            info!("Deleted resolver {}", id);
            Ok(Json(ApiResponse::success(DeleteResponse { deleted: true })))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete resolver {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ========== Priority Rule Handlers ==========

pub async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::db::PriorityRule>>>, StatusCode> {
    debug!("GET /api/v1/rules");

    match state.db.get_all_rules().await {
        Ok(rules) => {
            info!("Retrieved {} rules", rules.len());
            Ok(Json(ApiResponse::success(rules)))
        }
        Err(e) => {
            error!("Failed to get rules: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<crate::db::PriorityRule>>, StatusCode> {
    debug!("GET /api/v1/rules/{}", id);

    match state.db.get_rule_by_id(id).await {
        Ok(Some(rule)) => Ok(Json(ApiResponse::success(rule))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get rule {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_rule(
    State(state): State<AppState>,
    Json(payload): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CreateResponse>>), StatusCode> {
    debug!("POST /api/v1/rules - {:?}", payload);

    match state
        .db
        .create_rule(
            &payload.domain,
            payload.client_filter.as_deref(),
            &payload.target_ip,
            payload.active,
        )
        .await
    {
        Ok(id) => {
            info!(
                "Created rule {} - {} -> {} (filter: {:?}, active: {})",
                id, payload.domain, payload.target_ip, payload.client_filter, payload.active
            );
            Ok((
                StatusCode::CREATED,
                Json(ApiResponse::success(CreateResponse { id })),
            ))
        }
        Err(e) => {
            error!("Failed to create rule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateRuleRequest>,
) -> Result<Json<ApiResponse<UpdateResponse>>, StatusCode> {
    debug!("PUT /api/v1/rules/{} - {:?}", id, payload);

    match state
        .db
        .update_rule(
            id,
            payload.domain.as_deref(),
            payload.client_filter.as_ref().map(|opt| opt.as_deref()),
            payload.target_ip.as_deref(),
            payload.active,
        )
        .await
    {
        Ok(true) => {
            info!("Updated rule {}", id);
            Ok(Json(ApiResponse::success(UpdateResponse { updated: true })))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update rule {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn toggle_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<UpdateResponse>>, StatusCode> {
    debug!("PATCH /api/v1/rules/{}/toggle", id);

    match state.db.toggle_rule(id).await {
        Ok(true) => {
            info!("Toggled rule {}", id);
            Ok(Json(ApiResponse::success(UpdateResponse { updated: true })))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to toggle rule {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<DeleteResponse>>, StatusCode> {
    debug!("DELETE /api/v1/rules/{}", id);

    match state.db.delete_rule(id).await {
        Ok(true) => {
            info!("Deleted rule {}", id);
            Ok(Json(ApiResponse::success(DeleteResponse { deleted: true })))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete rule {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ========== Statistics Handlers ==========

pub async fn get_all_stats(
    State(state): State<AppState>,
    Query(params): Query<StatsQuery>,
) -> Result<Json<ApiResponse<Vec<crate::db::Statistic>>>, StatusCode> {
    debug!("GET /api/v1/stats?limit={}&offset={}", params.limit, params.offset);

    match state.db.get_all_stats(params.limit, params.offset).await {
        Ok(stats) => {
            info!("Retrieved {} statistics records", stats.len());
            Ok(Json(ApiResponse::success(stats)))
        }
        Err(e) => {
            error!("Failed to get statistics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_stats_for_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<crate::db::Statistic>>>, StatusCode> {
    debug!("GET /api/v1/stats/rules/{}", rule_id);

    match state.db.get_stats_for_rule(rule_id).await {
        Ok(stats) => {
            info!("Retrieved {} statistics for rule {}", stats.len(), rule_id);
            Ok(Json(ApiResponse::success(stats)))
        }
        Err(e) => {
            error!("Failed to get statistics for rule {}: {}", rule_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_stats_summary(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<StatsSummaryResponse>>, StatusCode> {
    debug!("GET /api/v1/stats/summary");

    match state.db.get_stats_summary().await {
        Ok(summary) => {
            info!("Retrieved statistics summary");
            Ok(Json(ApiResponse::success(StatsSummaryResponse {
                total_hits: summary.total_hits,
                unique_clients: summary.unique_clients,
                active_rules: summary.active_rules,
            })))
        }
        Err(e) => {
            error!("Failed to get statistics summary: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ========== Health Check Handler ==========

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// ========== Documentation Handler ==========

pub async fn api_docs() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("docs.html"))
}
