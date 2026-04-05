use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;

/// Spawns the app on a random port and returns (base_url, pool).
async fn spawn_app() -> (String, PgPool) {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    // Clean tables before each test run
    sqlx::query("DELETE FROM link_reports").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM podcasts").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM movies").execute(&pool).await.unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{port}");

    let app = hablemos_backend::create_router(pool.clone())
        .layer(tower_http::cors::CorsLayer::permissive());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (base_url, pool)
}

/// Insert a test podcast directly via SQL, returns its UUID.
async fn insert_test_podcast(pool: &PgPool) -> String {
    let row = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO podcasts (title, description, image_url, language, level, country, topic, url) \
         VALUES ('Test Pod', 'A test podcast', 'https://img.example.com/test.jpg', 'es', 'beginner', 'Mexico', 'grammar', 'https://example.com/pod') \
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    row.to_string()
}

// ─── Health ──────────────────────────────────────────────

#[tokio::test]
async fn health_check_returns_200() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let res = client.get(format!("{url}/health")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
}

// ─── GET /api/podcasts ───────────────────────────────────

#[tokio::test]
async fn list_podcasts_returns_empty_when_no_data() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let res = client.get(format!("{url}/api/podcasts")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["totalCount"], 0);
}

#[tokio::test]
async fn list_podcasts_returns_inserted_podcast() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    insert_test_podcast(&pool).await;

    let res = client.get(format!("{url}/api/podcasts")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Test Pod");
    assert_eq!(items[0]["language"], "es");
    assert_eq!(body["pagination"]["totalCount"], 1);
}

#[tokio::test]
async fn list_podcasts_excludes_archived_by_default() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    let id = insert_test_podcast(&pool).await;
    sqlx::query("UPDATE podcasts SET archived = true WHERE id = $1::UUID")
        .bind(&id)
        .execute(&pool)
        .await
        .unwrap();

    let res = client.get(format!("{url}/api/podcasts")).send().await.unwrap();
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 0);

    // With includeArchived=true
    let res = client
        .get(format!("{url}/api/podcasts?includeArchived=true"))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
}

// ─── GET /api/podcasts/:id ───────────────────────────────

#[tokio::test]
async fn get_podcast_by_id_returns_podcast() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    let id = insert_test_podcast(&pool).await;

    let res = client
        .get(format!("{url}/api/podcasts/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["title"], "Test Pod");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn get_podcast_by_id_returns_404_for_missing() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let fake_id = uuid::Uuid::new_v4();
    let res = client
        .get(format!("{url}/api/podcasts/{fake_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

// ─── POST /api/podcasts/:id/report ───────────────────────

#[tokio::test]
async fn report_dead_link_returns_201() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    let id = insert_test_podcast(&pool).await;

    let res = client
        .post(format!("{url}/api/podcasts/{id}/report"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["podcastId"], id);
}

#[tokio::test]
async fn report_dead_link_returns_404_for_missing_podcast() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let fake_id = uuid::Uuid::new_v4();
    let res = client
        .post(format!("{url}/api/podcasts/{fake_id}/report"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

// ─── Phase 2: POST /api/podcasts (create) ────────────────

#[tokio::test]
async fn create_podcast_returns_201() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let body = serde_json::json!({
        "title": "New Pod",
        "description": "A brand new podcast",
        "imageUrl": "https://img.example.com/new.jpg",
        "language": "en",
        "level": "advanced",
        "country": "Spain",
        "topic": "culture",
        "url": "https://example.com/new"
    });

    let res = client
        .post(format!("{url}/api/podcasts"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let podcast: Value = res.json().await.unwrap();
    assert_eq!(podcast["title"], "New Pod");
    assert_eq!(podcast["language"], "en");
    assert!(podcast["id"].as_str().is_some());
}

#[tokio::test]
async fn create_podcast_validates_input() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let body = serde_json::json!({
        "title": "",
        "description": "ok",
        "imageUrl": "not-a-url",
        "language": "invalid",
        "level": "beginner",
        "country": "Mexico",
        "topic": "grammar",
        "url": "https://example.com"
    });

    let res = client
        .post(format!("{url}/api/podcasts"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

// ─── PATCH /api/podcasts/:id ─────────────────────────────

#[tokio::test]
async fn update_podcast_partial() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_podcast(&pool).await;

    let body = serde_json::json!({"title": "Updated Title"});
    let res = client
        .patch(format!("{url}/api/podcasts/{id}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let podcast: Value = res.json().await.unwrap();
    assert_eq!(podcast["title"], "Updated Title");
    // Other fields unchanged
    assert_eq!(podcast["language"], "es");
}

#[tokio::test]
async fn update_podcast_returns_404_for_missing() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();
    let fake_id = uuid::Uuid::new_v4();

    let body = serde_json::json!({"title": "Nope"});
    let res = client
        .patch(format!("{url}/api/podcasts/{fake_id}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

// ─── DELETE /api/podcasts/:id ────────────────────────────

#[tokio::test]
async fn delete_podcast_returns_204() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_podcast(&pool).await;

    let res = client
        .delete(format!("{url}/api/podcasts/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Confirm it's gone
    let res = client
        .get(format!("{url}/api/podcasts/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_podcast_returns_404_for_missing() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();
    let fake_id = uuid::Uuid::new_v4();

    let res = client
        .delete(format!("{url}/api/podcasts/{fake_id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

// ─── POST /api/podcasts/:id/archive + /unarchive ────────

#[tokio::test]
async fn archive_and_unarchive_podcast() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_podcast(&pool).await;

    // Archive
    let res = client
        .post(format!("{url}/api/podcasts/{id}/archive"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["archived"], true);

    // Unarchive
    let res = client
        .post(format!("{url}/api/podcasts/{id}/unarchive"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["archived"], false);
}

// ─── GET /api/podcasts/:id/reports ───────────────────────

#[tokio::test]
async fn list_reports_for_podcast() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_podcast(&pool).await;

    // Create a report first
    client
        .post(format!("{url}/api/podcasts/{id}/report"))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{url}/api/podcasts/{id}/reports"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let reports: Vec<Value> = res.json().await.unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["podcastId"], id);
}

// ─── GET /api/podcasts/:id/reports/count ─────────────────

#[tokio::test]
async fn count_reports_for_podcast() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_podcast(&pool).await;

    client
        .post(format!("{url}/api/podcasts/{id}/report"))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{url}/api/podcasts/{id}/reports/count"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["count"], 1);
}

// ─── DELETE /api/podcasts/:id/reports ────────────────────

#[tokio::test]
async fn clear_reports_for_podcast() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_podcast(&pool).await;

    client
        .post(format!("{url}/api/podcasts/{id}/report"))
        .send()
        .await
        .unwrap();

    let res = client
        .delete(format!("{url}/api/podcasts/{id}/reports"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Confirm cleared
    let res = client
        .get(format!("{url}/api/podcasts/{id}/reports/count"))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["count"], 0);
}

// ─── GET /api/link-reports/counts ────────────────────────

#[tokio::test]
async fn get_all_report_counts() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_podcast(&pool).await;

    client
        .post(format!("{url}/api/podcasts/{id}/report"))
        .send()
        .await
        .unwrap();

    let res = client
        .get(format!("{url}/api/link-reports/counts"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let counts: Vec<Value> = res.json().await.unwrap();
    assert_eq!(counts.len(), 1);
    assert_eq!(counts[0]["podcastId"], id);
    assert_eq!(counts[0]["count"], 1);
}

// ─── Movie helpers ───────────────────────────────────────

async fn insert_test_movie(pool: &PgPool, language: &str) -> String {
    let row = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO movies (title, description, poster_url, audio_language, country, genre, release_year, url) \
         VALUES ('Test Movie', 'A test movie', 'https://img.example.com/poster.jpg', $1, 'Mexico', 'Drama', 2020, 'https://example.com/movie') \
         RETURNING id"
    )
    .bind(language)
    .fetch_one(pool)
    .await
    .unwrap();
    row.to_string()
}

// ─── GET /api/movies ─────────────────────────────────────

#[tokio::test]
async fn list_movies_returns_empty_when_no_data() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let res = client.get(format!("{url}/api/movies")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["totalCount"], 0);
}

#[tokio::test]
async fn list_movies_returns_inserted_movie() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    insert_test_movie(&pool, "es").await;

    let res = client.get(format!("{url}/api/movies")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Test Movie");
    assert_eq!(items[0]["audioLanguage"], "es");
}

#[tokio::test]
async fn list_movies_filters_by_language() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    insert_test_movie(&pool, "es").await;
    insert_test_movie(&pool, "en").await;

    let res = client
        .get(format!("{url}/api/movies?audioLanguage=es"))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["audioLanguage"], "es");
}

// ─── GET /api/movies/:id ─────────────────────────────────

#[tokio::test]
async fn get_movie_by_id_returns_movie() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    let id = insert_test_movie(&pool, "en").await;

    let res = client.get(format!("{url}/api/movies/{id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = res.json().await.unwrap();
    assert_eq!(body["title"], "Test Movie");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn get_movie_by_id_returns_404_for_missing() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let fake_id = uuid::Uuid::new_v4();
    let res = client.get(format!("{url}/api/movies/{fake_id}")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

// ─── POST /api/movies ────────────────────────────────────

#[tokio::test]
async fn create_movie_returns_201() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let body = serde_json::json!({
        "title": "New Movie",
        "description": "A brand new movie",
        "posterUrl": "https://img.example.com/new.jpg",
        "audioLanguage": "es",
        "country": "Spain",
        "genre": "Comedy",
        "releaseYear": 2023,
        "url": "https://example.com/new-movie"
    });

    let res = client
        .post(format!("{url}/api/movies"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let movie: Value = res.json().await.unwrap();
    assert_eq!(movie["title"], "New Movie");
    assert_eq!(movie["audioLanguage"], "es");
    assert_eq!(movie["releaseYear"], 2023);
}

#[tokio::test]
async fn create_movie_validates_input() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let body = serde_json::json!({
        "title": "",
        "description": "ok",
        "posterUrl": "not-a-url",
        "audioLanguage": "invalid",
        "country": "Mexico",
        "genre": "Drama",
        "releaseYear": 2020,
        "url": "https://example.com"
    });

    let res = client
        .post(format!("{url}/api/movies"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

// ─── DELETE /api/movies/:id ──────────────────────────────

#[tokio::test]
async fn delete_movie_returns_204() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();
    let id = insert_test_movie(&pool, "en").await;

    let res = client.delete(format!("{url}/api/movies/{id}")).send().await.unwrap();
    assert_eq!(res.status(), 204);

    // Confirm gone
    let res = client.get(format!("{url}/api/movies/{id}")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_movie_returns_404_for_missing() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();
    let fake_id = uuid::Uuid::new_v4();

    let res = client.delete(format!("{url}/api/movies/{fake_id}")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

// ─── DELETE /api/movies (bulk) ───────────────────────────

#[tokio::test]
async fn bulk_delete_movies_returns_204() {
    let (url, pool) = spawn_app().await;
    let client = Client::new();

    let id1 = insert_test_movie(&pool, "en").await;
    let id2 = insert_test_movie(&pool, "es").await;

    let body = serde_json::json!({ "ids": [id1, id2] });
    let res = client
        .delete(format!("{url}/api/movies"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Confirm both gone
    let res = client.get(format!("{url}/api/movies")).send().await.unwrap();
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn bulk_delete_movies_rejects_empty_ids() {
    let (url, _pool) = spawn_app().await;
    let client = Client::new();

    let body = serde_json::json!({ "ids": [] });
    let res = client
        .delete(format!("{url}/api/movies"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}
