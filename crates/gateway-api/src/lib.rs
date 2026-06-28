use std::sync::Arc;

use gateway_core::router::Router;

pub mod handlers;
pub mod middleware;

/// Shared application state for the gateway API
pub struct AppState {
    /// The request router that dispatches to providers
    pub router: Arc<Router>,
}
