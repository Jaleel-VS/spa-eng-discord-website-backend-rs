mod health;
mod link_report;
mod movie;
mod podcast;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

pub fn create_router(pool: PgPool) -> Router {
    let api = Router::new()
        .route("/podcasts", get(podcast::list).post(podcast::create))
        .route(
            "/podcasts/{id}",
            get(podcast::get_by_id)
                .patch(podcast::update)
                .delete(podcast::delete),
        )
        .route("/podcasts/{id}/archive", post(podcast::archive))
        .route("/podcasts/{id}/unarchive", post(podcast::unarchive))
        .route("/podcasts/{id}/report", post(link_report::report))
        .route(
            "/podcasts/{id}/reports",
            get(link_report::list_by_podcast).delete(link_report::clear_reports),
        )
        .route("/podcasts/{id}/reports/count", get(link_report::count_by_podcast))
        .route("/link-reports/counts", get(link_report::get_all_counts))
        .route("/movies", get(movie::list).post(movie::create).delete(movie::bulk_delete))
        .route("/movies/{id}", get(movie::get_by_id).delete(movie::delete));

    Router::new()
        .route("/health", get(health::health_check))
        .nest("/api", api)
        .with_state(pool)
}
