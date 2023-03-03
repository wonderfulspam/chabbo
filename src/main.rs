use anyhow::Result;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use chabbo::backends::{deta::DetaService, local::LocalService, Backend};
use markov::Chain;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const INDEX_HTML: &str = include_str!("../static/index.html");
const CORPUS_HTML: &str = include_str!("../static/corpus.html");

#[derive(Deserialize)]
struct Config {
    // Will not be set in local env
    deta_project_key: Option<String>,
    // May not be set in local env but is always needed
    // Thus, we set a default
    #[serde(default = "default_port")]
    port: u16,
}

fn default_port() -> u16 {
    3000
}

#[derive(Clone)]
struct AppState {
    chain: Arc<Mutex<Chain<String>>>,
    backend: Arc<Box<dyn Backend>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(config) = envy::from_env::<Config>() else { return Err("Please provide all necessary envvars".into()) };
    let is_deta = config.deta_project_key.is_some();

    // Setup tracing
    let fmt_layer = if is_deta {
        // Deta's log overview can't handle ANSI escape codes
        tracing_subscriber::fmt::layer().with_ansi(false)
    } else {
        tracing_subscriber::fmt::layer()
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "chabbo=debug,tower_http=debug".into()),
        ))
        .with(fmt_layer)
        .init();

    // Choose storage backend based on env
    let backend: Box<dyn Backend> = if let Some(deta_project_key) = config.deta_project_key {
        debug!("creating Deta service");
        Box::new(DetaService::new(deta_project_key))
    } else {
        debug!("using local backend");
        Box::<LocalService>::default()
    };
    debug!("initialized backend");

    // Setup initial markov chain
    let text = backend.get_initial_corpus()?;
    let chain = chabbo::get_chain_from_text(&text);

    // Create app state
    let state = AppState {
        chain: Arc::new(Mutex::new(chain)),
        backend: Arc::new(backend),
    };

    // Setup routes and layers
    let app = Router::new()
        .route("/", get(index))
        .route("/", post(markov_response))
        .route("/corpus", get(corpus_page))
        .route("/corpus/active", get(active_corpus_name))
        .route("/corpus/list", get(list_corpora))
        .route("/corpus", put(upload_corpus))
        .route("/corpus", post(choose_corpus))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    // Run!
    let port = config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

// Not just static HTML, but statically compiled HTML!
async fn index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

async fn corpus_page() -> impl IntoResponse {
    Html(CORPUS_HTML)
}

async fn active_corpus_name(
    State(state): State<AppState>,
) -> Result<Json<ActiveCorpusResponse>, StatusCode> {
    let name = state
        .backend
        .get_active_corpus_name()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ActiveCorpusResponse { name }))
}

// Output from `active_corpus_name`
#[derive(Serialize)]
struct ActiveCorpusResponse {
    name: String,
}

// Generate markov response based on prompt
async fn markov_response(
    State(state): State<AppState>,
    Json(payload): Json<MarkovRequest>,
) -> Json<MarkovResponse> {
    let guard = state.chain.lock().await;
    let generated_string = if payload.input.is_empty() {
        guard.generate_str()
    } else {
        guard.generate_str_from_token(&payload.input.to_lowercase())
    };
    drop(guard);

    let response = if generated_string.is_empty() {
        format!("No string found for {}", &payload.input)
    } else {
        generated_string
    };
    Json(MarkovResponse { response })
}

// Input for `markov_response`
#[derive(Deserialize)]
struct MarkovRequest {
    input: String,
}

// Output from `markov_response`
#[derive(Serialize)]
struct MarkovResponse {
    response: String,
}

async fn upload_corpus(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(), StatusCode> {
    // Get upload field
    let Some(field) = multipart.next_field().await.unwrap() else {
        return Err(StatusCode::BAD_REQUEST);
    };

    // Read name and bytes
    let name = field.file_name().unwrap().to_string();
    let data = field.bytes().await.unwrap();
    debug!("length of `{}` is {} bytes", name, data.len());

    // Upload file
    let name = state
        .backend
        .upload_file(&name, &data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Replace chain
    let text = String::from_utf8_lossy(&data);
    replace_chain(State(state), &name, text.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

#[derive(Deserialize)]
struct ChooseCorpusRequest {
    corpus: String,
}

async fn choose_corpus(
    State(state): State<AppState>,
    Json(payload): Json<ChooseCorpusRequest>,
) -> Result<(), StatusCode> {
    // Load corpus
    let corpus_name = payload.corpus;
    let text = match state.backend.get_file_contents(&corpus_name) {
        Ok(corpus) => corpus,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Replace chain
    replace_chain(State(state), &corpus_name, &text)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

async fn replace_chain(
    State(state): State<AppState>,
    corpus_name: &str,
    text: &str,
) -> Result<String> {
    let chain = chabbo::get_chain_from_text(text);
    let mut guard = state.chain.lock().await;
    *guard = chain;
    drop(guard);

    // Set active corpus
    state.backend.set_active_corpus_name(corpus_name)
}

#[derive(Serialize)]
struct Corpus {
    name: String,
    is_active: bool,
}

async fn list_corpora(State(state): State<AppState>) -> Result<Json<Vec<Corpus>>, StatusCode> {
    let mut corpora = vec![];
    let active_corpus = state
        .backend
        .get_active_corpus_name()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    corpora.push("Default".to_string());

    let mut files = match state.backend.list_files() {
        Ok(files) => files,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    corpora.append(&mut files);
    let res = corpora
        .iter()
        .map(|c| {
            let is_active = active_corpus.eq(c);
            Corpus {
                name: c.to_string(),
                is_active,
            }
        })
        .collect();
    Ok(Json(res))
}
