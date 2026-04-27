use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures_util::{Stream, StreamExt};
use serde::Serialize;
use std::{convert::Infallible, net::SocketAddr};
use tokio::{net::TcpListener, sync::broadcast};
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    events: broadcast::Sender<String>,
}

#[derive(Serialize)]
struct TaskEvent {
    event_type: &'static str,
    task_id: Uuid,
}

async fn ws(upgrade: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| handle_socket(socket, state.events.subscribe()))
}

async fn handle_socket(mut socket: WebSocket, mut rx: broadcast::Receiver<String>) {
    if socket
        .send(Message::Text("{\"event_type\":\"connected\"}".into()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(event) => {
                        if socket.send(Message::Text(event.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => break,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            message = socket.recv() => {
                match message {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

async fn sse_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.events.subscribe()).filter_map(|msg| async move {
        match msg {
            Ok(data) => Some(Ok(Event::default().event("task_event").data(data))),
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn demo_task_created(State(state): State<AppState>) -> Json<serde_json::Value> {
    let event = TaskEvent {
        event_type: "task_created",
        task_id: Uuid::new_v4(),
    };
    let json = serde_json::to_string(&event).expect("event serialization failed");
    let _ = state.events.send(json.clone());
    Json(serde_json::json!({ "sent": json }))
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/ws", get(ws))
        .route("/events", get(sse_events))
        .route("/demo/task-created", post(demo_task_created))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let (tx, _rx) = broadcast::channel(1024);
    let state = AppState { events: tx };

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
