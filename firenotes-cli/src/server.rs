use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use firenotes_core::{Document, Parser, Storage, Templates};

#[derive(Clone)]
struct AppState {
    storage: Arc<Storage>,
    parser: Arc<Parser>,
}

#[derive(Serialize, Deserialize)]
struct CreateDocumentRequest {
    id: Option<String>,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct UpdateDocumentRequest {
    title: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct ParseRequest {
    markdown: String,
}

#[derive(Serialize, Deserialize)]
struct ParseResponse {
    html: String,
    terminal: String,
    plain: String,
}

#[derive(Serialize, Deserialize)]
struct SearchParams {
    q: String,
}

#[derive(Serialize, Deserialize)]
struct ReplaceAllRequest {
    query: String,
    replacement: String,
}

pub async fn run_server(port: u16) -> Result<()> {
    let storage = Arc::new(Storage::default()?);
    let parser = Arc::new(Parser::new());

    let state = AppState {
        storage,
        parser,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/documents", get(list_documents).post(create_document))
        .route("/api/documents/:id", get(get_document).put(update_document).delete(delete_document))
        .route("/api/documents/:id/parse", post(parse_document))
        .route("/api/documents/:id/restore/:index", post(restore_document_version))
        .route("/api/search", get(search_documents))
        .route("/api/search/replace", post(replace_all_documents))
        .route("/api/templates", get(list_templates))
        .route("/api/templates/:id", post(create_from_template))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🚀 Server running on http://0.0.0.0:{}", port);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn list_documents(State(state): State<AppState>) -> impl IntoResponse {
    match state.storage.list_all() {
        Ok(documents) => Json(documents).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() }))
        ).into_response(),
    }
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => {
            match state.storage.load(uuid) {
                Ok(Some(document)) => Json(document).into_response(),
                Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Document not found" }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid UUID" }))).into_response(),
    }
}

async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> impl IntoResponse {
    let mut document = match req.id {
        Some(id) => match uuid::Uuid::parse_str(&id) {
            Ok(uuid) => Document::with_id(uuid, req.title, req.content),
            Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid UUID" }))).into_response(),
        },
        None => Document::new(req.title, req.content),
    };
    if let Some(tags) = req.tags {
        document.set_tags(tags);
    }
    
    match state.storage.save(&document) {
        Ok(_) => (StatusCode::CREATED, Json(document)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDocumentRequest>,
) -> impl IntoResponse {
    match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => {
            match state.storage.load(uuid) {
                Ok(Some(mut document)) => {
                    if let Some(title) = req.title {
                        document.update_title(title);
                    }
                    if let Some(content) = req.content {
                        document.update_content(content);
                    }
                    if let Some(tags) = req.tags {
                        document.set_tags(tags);
                    }
                    
                    match state.storage.save(&document) {
                        Ok(_) => Json(document).into_response(),
                        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
                    }
                }
                Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Document not found" }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid UUID" }))).into_response(),
    }
}

async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => {
            match state.storage.delete(uuid) {
                Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid UUID" }))).into_response(),
    }
}

async fn parse_document(
    State(state): State<AppState>,
    Path(_id): Path<String>,
    Json(req): Json<ParseRequest>,
) -> impl IntoResponse {
    let html = match state.parser.parse_to_html(&req.markdown) {
        Ok(html) => html,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    let terminal = match state.parser.parse_to_terminal(&req.markdown) {
        Ok(terminal) => terminal,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    let plain = match state.parser.parse_to_plain(&req.markdown) {
        Ok(plain) => plain,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    Json(ParseResponse { html, terminal, plain }).into_response()
}

async fn restore_document_version(
    State(state): State<AppState>,
    Path((id, index)): Path<(String, usize)>,
) -> impl IntoResponse {
    match uuid::Uuid::parse_str(&id) {
        Ok(uuid) => {
            match state.storage.load(uuid) {
                Ok(Some(mut document)) => {
                    if !document.restore_version(index) {
                        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "History snapshot not found" }))).into_response();
                    }

                    match state.storage.save(&document) {
                        Ok(_) => Json(document).into_response(),
                        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
                    }
                }
                Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Document not found" }))).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid UUID" }))).into_response(),
    }
}

async fn search_documents(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    match state.storage.search(&params.q) {
        Ok(documents) => Json(documents).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn replace_all_documents(
    State(state): State<AppState>,
    Json(req): Json<ReplaceAllRequest>,
) -> impl IntoResponse {
    match state.storage.replace_all(&req.query, &req.replacement) {
        Ok(documents) => Json(documents).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

async fn list_templates() -> impl IntoResponse {
    Json(Templates::all()).into_response()
}

async fn create_from_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(document) = Templates::create_document(&id) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Template not found" }))).into_response();
    };

    match state.storage.save(&document) {
        Ok(_) => (StatusCode::CREATED, Json(document)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}