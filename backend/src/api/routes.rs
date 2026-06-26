use axum::{
    routing::{get, patch},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;

use crate::db::Database;
use crate::dns::ResolverCacheHandle;
use crate::rules::RuleCacheHandle;
use super::handlers::*;

pub fn create_router(
    db: Arc<Database>,
    rule_cache: RuleCacheHandle,
    resolver_cache: ResolverCacheHandle,
    enable_cors: bool,
) -> Router {
    let state = AppState {
        db,
        rule_cache,
        resolver_cache,
    };

    // Create API routes
    let api_routes = Router::new()
        // Health check
        .route("/health", get(health_check))
        // Documentation
        .route("/docs", get(api_docs))
        // Upstream resolvers
        .route("/resolvers", get(list_resolvers).post(create_resolver))
        .route(
            "/resolvers/{id}",
            get(get_resolver)
                .put(update_resolver)
                .delete(delete_resolver),
        )
        // Priority rules
        .route("/rules", get(list_rules).post(create_rule))
        .route(
            "/rules/{id}",
            get(get_rule).put(update_rule).delete(delete_rule),
        )
        .route("/rules/{id}/toggle", patch(toggle_rule))
        // Statistics
        .route("/stats", get(get_all_stats))
        .route("/stats/rules/{rule_id}", get(get_stats_for_rule))
        .route("/stats/summary", get(get_stats_summary))
        .with_state(state);

    // Nest under /api/v1
    let app = Router::new().nest("/api/v1", api_routes);

    // Add CORS if enabled
    if enable_cors {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        app.layer(cors)
    } else {
        app
    }
}
