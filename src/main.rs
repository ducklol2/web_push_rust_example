use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};
use tokio::spawn;
use tower_http::services::ServeDir;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use web_push::*;

struct AppState {
    subscriptions: Arc<Mutex<HashMap<String, SubscriptionInfo>>>,
    signature_builder: PartialVapidSignatureBuilder,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        subscriptions: Arc::new(Mutex::new(HashMap::new())),
        signature_builder: VapidSignatureBuilder::from_pem_no_sub(
            File::open("private_key.pem").expect("Could not open private_key.pem"),
        )
        .unwrap(),
    });

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let app = Router::new()
        .route("/subscribe", post(handle_subscribe))
        .route("/push", post(handle_push))
        .fallback_service(ServeDir::new("static"))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_subscribe(
    State(state): State<Arc<AppState>>,
    Json(info): Json<SubscriptionInfo>,
) -> StatusCode {
    println!("New subscription: {:?}", info);
    let mut subs = state.subscriptions.lock().expect("poisoned?!");
    subs.insert(info.endpoint.clone(), info);
    StatusCode::CREATED
}

#[axum::debug_handler]
async fn handle_push(State(state): State<Arc<AppState>>) -> StatusCode {
    println!("Pushing notifications: {:?}", state.subscriptions);

    for sub in state.subscriptions.lock().unwrap().values() {
        let builder2 = state.signature_builder.clone();
        let sub2 = sub.clone();
        spawn(async move {
            push_notification(builder2, sub2)
                .await
                .expect("failed to push");
        });
    }
    StatusCode::CREATED
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PushData {
    notification: Notification,
    app_badge: u32,
}

// Matches https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerRegistration/showNotification.
#[derive(Serialize, Deserialize)]
struct Notification {
    title: String,
    options: NotificationOptions,
}

#[derive(Serialize, Deserialize)]
struct NotificationOptions {
    body: String,
    badge: String,
    icon: String,
    timestamp: Option<u64>,
}

async fn push_notification(
    signature_builder: PartialVapidSignatureBuilder,
    info: SubscriptionInfo,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut builder = WebPushMessageBuilder::new(&info);
    builder.set_vapid_signature(signature_builder.add_sub_info(&info).build().unwrap());
    // builder.set_urgency(Urgency::High);

    let push_data = PushData {
        app_badge: 1234,
        notification: Notification {
            title: "Hello from Rust!".to_owned(),
            options: NotificationOptions {
                body: "This is a Rust-y body".to_owned(),
                badge: "badge.png".to_owned(),
                icon: "icon.png".to_owned(),
                timestamp: Option::None,
            },
        },
    };
    let mut bytes: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut bytes, &push_data).unwrap();
    builder.set_payload(ContentEncoding::Aes128Gcm, &bytes);

    IsahcWebPushClient::new()?.send(builder.build()?).await?;
    Ok(())
}
