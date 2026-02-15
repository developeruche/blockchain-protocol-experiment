use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use bytes::Bytes;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::UnixListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    data_100mb: Bytes,
    data_300mb: Bytes,
    data_500mb: Bytes,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct RpcRequest {
    method: String,
    params: Vec<serde_json::Value>,
    id: u64,
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

const SECRET_KEY: &[u8] = b"super_secret_key";

use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Pre-generating data...");
    let data_100mb = generate_random_payload(100 * 1024 * 1024);
    let data_300mb = generate_random_payload(300 * 1024 * 1024);
    let data_500mb = generate_random_payload(500 * 1024 * 1024);
    tracing::info!("Data generation complete.");

    let state = AppState {
        data_100mb,
        data_300mb,
        data_500mb,
    };

    let app = Router::new()
        .route("/", post(rpc_handler))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state);

    let socket_path = "/tmp/benchmark.sock";
    if std::fs::metadata(socket_path).is_ok() {
        std::fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    tracing::info!("Server listening on {}", socket_path);

    loop {
        let (socket, _addr) = listener.accept().await?;
        let app = app.clone();

        tokio::spawn(async move {
            let socket = TokioIo::new(socket);

            let service = hyper::service::service_fn(move |req: Request<Incoming>| {
                let app = app.clone();
                async move {
                    let req = req.map(axum::body::Body::new);
                    app.oneshot(req).await
                }
            });

            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, service)
                .await
            {
                tracing::error!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

fn generate_random_payload(size: usize) -> Bytes {
    let mut rng = rand::thread_rng();
    let mut data = vec![0u8; size];
    rng.fill(&mut data[..]);
    Bytes::from(data)
}

async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(auth_header) if auth_header.starts_with("Bearer ") => {
            let token = &auth_header[7..];
            let validation = Validation::new(Algorithm::HS256);
            match decode::<Claims>(token, &DecodingKey::from_secret(SECRET_KEY), &validation) {
                Ok(_) => Ok(next.run(req).await),
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn rpc_handler(
    State(state): State<AppState>,
    Json(payload): Json<RpcRequest>,
) -> Result<impl IntoResponse, AppError> {
    match payload.method.as_str() {
        "get_data" => {
            let size_str = payload
                .params
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("100mb");
            let data = match size_str {
                "100mb" => &state.data_100mb,
                "300mb" => &state.data_300mb,
                "500mb" => &state.data_500mb,
                _ => return Err(anyhow::anyhow!("Invalid size request").into()),
            };

            Ok((
                [(header::CONTENT_TYPE, "application/octet-stream")],
                data.clone(),
            ))
        }
        _ => Err(anyhow::anyhow!("Method not found").into()),
    }
}
