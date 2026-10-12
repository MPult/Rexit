//! Local web viewer for chat exports (`rexit view`)
//!
//! Loads an existing export from disk and serves it on localhost together with a small
//! embedded web app. Nothing is fetched from Reddit and nothing leaves the machine.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use console::style;
use serde::{Deserialize, Serialize};

use crate::export::Metadata;
use crate::ReAPI::{Content, Room};

const INDEX_HTML: &str = include_str!("assets/index.html");
const APP_JS: &str = include_str!("assets/app.js");
const STYLE_CSS: &str = include_str!("assets/style.css");

/// Messages returned per page when none is requested
const DEFAULT_PAGE: usize = 200;
/// Maximum number of search results
const MAX_RESULTS: usize = 200;

/// An export loaded into memory
#[derive(Debug, Default)]
pub struct Export {
    user_id: Option<String>,
    exported_at: Option<DateTime<Utc>>,
    rooms: Vec<ViewRoom>,
    images_dir: PathBuf,
}

#[derive(Debug, Serialize)]
struct ViewRoom {
    key: String,
    name: String,
    members: Vec<String>,
    #[serde(skip)]
    messages: Vec<ViewMessage>,
}

#[derive(Debug, Clone, Serialize)]
struct ViewMessage {
    id: String,
    author: String,
    sender: String,
    timestamp: DateTime<Utc>,
    mine: bool,
    /// "text", "image" or "deleted"
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    /// Url of the downloaded image, if it was downloaded
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

impl ViewRoom {
    fn last(&self) -> Option<&ViewMessage> {
        self.messages.last()
    }
}

/// Load an export folder (the `--out` folder of `rexit messages`)
pub fn load(out: &Path) -> Result<Export, String> {
    let messages_dir = out.join("messages");
    if !messages_dir.is_dir() {
        return Err(format!(
            "No chat export found in {}; run `rexit messages` first",
            out.display()
        ));
    }

    // Schema v2 exports have a metadata.json; v1 exports are just the room files
    let metadata: Option<Metadata> = std::fs::read_to_string(messages_dir.join("metadata.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok());

    let files: Vec<String> = match &metadata {
        Some(metadata) => metadata.rooms.iter().map(|r| r.file.clone()).collect(),
        None => {
            let mut files: Vec<String> = std::fs::read_dir(&messages_dir)
                .map_err(|e| e.to_string())?
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let path = entry.path();
                    (path.extension()? == "json" && path.file_stem()? != "metadata")
                        .then(|| path.file_stem()?.to_str().map(str::to_owned))?
                })
                .collect();
            files.sort();
            files
        }
    };

    let images_dir = messages_dir.join("images");
    let images = index_images(&images_dir);
    let user_id = metadata.as_ref().and_then(|m| m.user_id.clone());

    let mut rooms = vec![];
    for file in files {
        let path = messages_dir.join(format!("{file}.json"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            log::warn!("Missing room file {}", path.display());
            continue;
        };
        match serde_json::from_str::<Room>(&text) {
            Ok(room) => rooms.push(to_view_room(file, room, user_id.as_deref(), &images)),
            Err(e) => log::warn!("Could not read {}: {e}", path.display()),
        }
    }

    // Most recent conversation first
    rooms.sort_by_key(|room| std::cmp::Reverse(room.last().map(|m| m.timestamp)));

    Ok(Export {
        user_id,
        exported_at: metadata.map(|m| m.exported_at),
        rooms,
        images_dir,
    })
}

/// Maps image ids (file stems) to file names in the images folder
fn index_images(dir: &Path) -> HashMap<String, String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let name = entry.file_name().to_str()?.to_owned();
                    let stem = Path::new(&name).file_stem()?.to_str()?.to_owned();
                    Some((stem, name))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn to_view_room(
    key: String,
    room: Room,
    user_id: Option<&str>,
    images: &HashMap<String, String>,
) -> ViewRoom {
    let mut messages: Vec<ViewMessage> = room
        .messages
        .unwrap_or_default()
        .into_iter()
        .map(|message| {
            let mine = user_id.is_some_and(|me| me == message.sender);
            let (kind, text, image) = match message.content {
                // v1 exports stored image urls as text
                Content::Image(url) | Content::Message(url) if url.starts_with("mxc://") => {
                    let id = url.rsplit('/').next().unwrap_or_default();
                    ("image", None, images.get(id).map(|f| format!("/media/{f}")))
                }
                Content::Message(text) => ("text", Some(text), None),
                Content::Image(_) => ("image", None, None),
                Content::Deleted => ("deleted", None, None),
            };
            ViewMessage {
                id: message.id,
                author: message.author,
                sender: message.sender,
                timestamp: message.timestamp,
                mine,
                kind,
                text,
                image,
            }
        })
        .collect();
    // v1 exports are newest first
    messages.sort_by_key(|m| m.timestamp);

    // Other people in the chat, from the member list or else from the messages
    let mut others: Vec<String> = room
        .members
        .iter()
        .filter(|m| user_id != Some(m.id.as_str()))
        .map(|m| m.displayname.clone())
        .collect();
    if others.is_empty() {
        for message in messages.iter().filter(|m| !m.mine) {
            if message.author != "N/A" && !others.contains(&message.author) {
                others.push(message.author.clone());
            }
        }
    }
    others.retain(|name| !name.is_empty());

    let name = room
        .name
        .filter(|name| !name.is_empty())
        .or_else(|| (!others.is_empty()).then(|| others.join(", ")))
        .unwrap_or_else(|| "Unknown chat".to_owned());

    ViewRoom {
        key,
        name,
        members: others,
        messages,
    }
}

/// Build the web app
pub fn router(export: Export) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { asset(INDEX_HTML, "text/html; charset=utf-8") }),
        )
        .route(
            "/app.js",
            get(|| async { asset(APP_JS, "text/javascript") }),
        )
        .route("/style.css", get(|| async { asset(STYLE_CSS, "text/css") }))
        .route("/api/info", get(info))
        .route("/api/rooms", get(rooms))
        .route("/api/rooms/{key}/messages", get(messages))
        .route("/api/search", get(search))
        .route("/media/{file}", get(media))
        .with_state(Arc::new(export))
}

/// Load the export and serve it until Ctrl+C
pub async fn serve(out: PathBuf, port: u16, open: bool) {
    let export = match load(&out) {
        Ok(export) => export,
        Err(e) => {
            println!("{}", style(e).red().bold());
            crate::exit!(1);
        }
    };
    let room_count = export.rooms.len();

    let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
        Ok(listener) => listener,
        Err(e) => {
            println!(
                "{}",
                style(format!("Could not listen on port {port}: {e}"))
                    .red()
                    .bold()
            );
            crate::exit!(1);
        }
    };
    let url = format!("http://{}", listener.local_addr().unwrap());

    println!(
        "Viewing {room_count} chat(s) at {}  (press Ctrl+C to stop)",
        style(&url).cyan().bold()
    );
    if open && webbrowser::open(&url).is_err() {
        println!("Could not open a browser; open the link above yourself");
    }

    axum::serve(listener, router(export))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await
        .unwrap();
}

fn asset(body: &'static str, content_type: &'static str) -> Response {
    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}

#[derive(Serialize)]
struct Info {
    user_id: Option<String>,
    exported_at: Option<DateTime<Utc>>,
    rooms: usize,
    messages: usize,
}

async fn info(State(export): State<Arc<Export>>) -> Json<Info> {
    Json(Info {
        user_id: export.user_id.clone(),
        exported_at: export.exported_at,
        rooms: export.rooms.len(),
        messages: export.rooms.iter().map(|r| r.messages.len()).sum(),
    })
}

#[derive(Serialize)]
struct RoomSummary<'a> {
    #[serde(flatten)]
    room: &'a ViewRoom,
    count: usize,
    last: Option<&'a ViewMessage>,
}

async fn rooms(State(export): State<Arc<Export>>) -> Response {
    let summaries: Vec<RoomSummary> = export
        .rooms
        .iter()
        .map(|room| RoomSummary {
            room,
            count: room.messages.len(),
            last: room.last(),
        })
        .collect();
    Json(summaries).into_response()
}

#[derive(Deserialize)]
struct PageQuery {
    /// Return messages before this index (default: the end)
    before: Option<usize>,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct Page<'a> {
    total: usize,
    /// Index of the first returned message
    start: usize,
    messages: &'a [ViewMessage],
}

async fn messages(
    State(export): State<Arc<Export>>,
    UrlPath(key): UrlPath<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    let Some(room) = export.rooms.iter().find(|r| r.key == key) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let total = room.messages.len();
    let end = query.before.unwrap_or(total).min(total);
    let start = end.saturating_sub(query.limit.unwrap_or(DEFAULT_PAGE).max(1));
    Json(Page {
        total,
        start,
        messages: &room.messages[start..end],
    })
    .into_response()
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
struct SearchHit<'a> {
    room: &'a str,
    room_name: &'a str,
    index: usize,
    message: &'a ViewMessage,
}

async fn search(State(export): State<Arc<Export>>, Query(query): Query<SearchQuery>) -> Response {
    let needle = query.q.trim().to_lowercase();
    if needle.is_empty() {
        return Json(Vec::<SearchHit>::new()).into_response();
    }
    let mut hits = vec![];
    'rooms: for room in &export.rooms {
        // Newest matches first
        for (index, message) in room.messages.iter().enumerate().rev() {
            let matches = message
                .text
                .as_ref()
                .is_some_and(|text| text.to_lowercase().contains(&needle));
            if matches {
                hits.push(SearchHit {
                    room: &room.key,
                    room_name: &room.name,
                    index,
                    message,
                });
                if hits.len() >= MAX_RESULTS {
                    break 'rooms;
                }
            }
        }
    }
    hits.sort_by_key(|hit| std::cmp::Reverse(hit.message.timestamp));
    Json(hits).into_response()
}

async fn media(State(export): State<Arc<Export>>, UrlPath(file): UrlPath<String>) -> Response {
    // Only plain file names from the images folder
    let valid = !file.is_empty()
        && !file.starts_with('.')
        && file
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if !valid {
        return StatusCode::NOT_FOUND.into_response();
    }

    let content_type = match Path::new(&file).extension().and_then(|e| e.to_str()) {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };

    match tokio::fs::read(export.images_dir.join(&file)).await {
        Ok(data) => (
            [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "max-age=86400"),
            ],
            data,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    const ME: &str = "@t2_me:reddit.com";

    /// Writes a small v2 export with one DM and returns its folder
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let messages = dir.path().join("messages");
        std::fs::create_dir_all(messages.join("images")).unwrap();
        std::fs::write(messages.join("images/abc123.png"), b"png").unwrap();
        std::fs::write(
            messages.join("metadata.json"),
            json!({
                "schema_version": 2,
                "exported_at": "2026-10-12T12:00:00Z",
                "user_id": ME,
                "rooms": [{"id": "!room1:reddit.com", "file": "room1"}]
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            messages.join("room1.json"),
            json!({
                "id": "!room1:reddit.com",
                "name": null,
                "members": [
                    {"id": ME, "displayname": "me"},
                    {"id": "@t2_friend:reddit.com", "displayname": "friend"}
                ],
                "messages": [
                    {"id": "$2", "author": "me", "sender": ME,
                     "timestamp": "2026-10-01T10:01:00Z", "content": {"Image": "mxc://reddit.com/abc123"}},
                    {"id": "$1", "author": "friend", "sender": "@t2_friend:reddit.com",
                     "timestamp": "2026-10-01T10:00:00Z", "content": {"Message": "Hello there"}},
                    {"id": "$3", "author": "friend", "sender": "@t2_friend:reddit.com",
                     "timestamp": "2026-10-01T10:02:00Z", "content": "Deleted"}
                ]
            })
            .to_string(),
        )
        .unwrap();
        dir
    }

    async fn get(dir: &tempfile::TempDir, uri: &str) -> (StatusCode, Vec<u8>) {
        let export = super::load(dir.path()).unwrap();
        let response = super::router(export)
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, body.to_vec())
    }

    async fn get_json(dir: &tempfile::TempDir, uri: &str) -> Value {
        let (status, body) = get(dir, uri).await;
        assert_eq!(status, StatusCode::OK);
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn rooms_are_named_after_the_other_person() {
        let dir = fixture();
        let rooms = get_json(&dir, "/api/rooms").await;

        assert_eq!(rooms[0]["name"], "friend");
        assert_eq!(rooms[0]["count"], 3);
        assert_eq!(rooms[0]["last"]["kind"], "deleted");
    }

    #[tokio::test]
    async fn messages_are_sorted_and_typed() {
        let dir = fixture();
        let page = get_json(&dir, "/api/rooms/room1/messages").await;
        let messages = page["messages"].as_array().unwrap();

        assert_eq!(page["total"], 3);
        assert_eq!(messages[0]["text"], "Hello there");
        assert_eq!(messages[0]["mine"], false);
        assert_eq!(messages[1]["kind"], "image");
        assert_eq!(messages[1]["image"], "/media/abc123.png");
        assert_eq!(messages[1]["mine"], true);
    }

    #[tokio::test]
    async fn messages_are_paged_from_the_end() {
        let dir = fixture();
        let page = get_json(&dir, "/api/rooms/room1/messages?before=3&limit=2").await;

        assert_eq!(page["start"], 1);
        assert_eq!(page["messages"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn search_finds_text() {
        let dir = fixture();
        let hits = get_json(&dir, "/api/search?q=hello").await;

        assert_eq!(hits.as_array().unwrap().len(), 1);
        assert_eq!(hits[0]["room"], "room1");
        assert_eq!(hits[0]["index"], 0);
    }

    #[tokio::test]
    async fn media_only_serves_image_files() {
        let dir = fixture();
        std::fs::write(dir.path().join("secret.txt"), b"secret").unwrap();

        assert_eq!(get(&dir, "/media/abc123.png").await.0, StatusCode::OK);
        assert_eq!(
            get(&dir, "/media/..%2F..%2Fsecret.txt").await.0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(get(&dir, "/media/.hidden").await.0, StatusCode::NOT_FOUND);
    }

    #[test]
    fn v1_exports_still_load() {
        let dir = tempfile::tempdir().unwrap();
        let messages = dir.path().join("messages");
        std::fs::create_dir_all(&messages).unwrap();
        std::fs::write(
            messages.join("abc.json"),
            json!({
                "id": "!abc:reddit.com",
                "messages": [
                    {"author": "friend", "timestamp": "2024-01-02T00:00:00Z", "content": {"Message": "newer"}},
                    {"author": "friend", "timestamp": "2024-01-01T00:00:00Z", "content": {"Message": "mxc://reddit.com/img"}}
                ]
            })
            .to_string(),
        )
        .unwrap();

        let export = super::load(dir.path()).unwrap();
        let room = &export.rooms[0];

        assert_eq!(room.name, "friend");
        assert_eq!(room.messages[0].kind, "image");
        assert_eq!(room.messages[1].text.as_deref(), Some("newer"));
    }
}
