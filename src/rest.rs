use serde::Serialize;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

pub const REST_BIND_ADDRESS: &str = "0.0.0.0:7080";

#[derive(Debug, Clone)]
pub enum RestCommand {
    Status(Sender<RestReply>),
    SetAutomix(bool, Sender<RestReply>),
    DeckPlay(Sender<RestReply>),
    DeckPause(Sender<RestReply>),
    DeckPlayPause(Sender<RestReply>),
    DeckStop(Sender<RestReply>),
    DeckRestart(Sender<RestReply>),
    DeckSeek(i64, Sender<RestReply>),
    DeckQueuePlay(usize, Sender<RestReply>),
    DeckPreviewPlay(usize, Sender<RestReply>),
    DeckPreviewToggle(usize, Sender<RestReply>),
    DeckPreviewStop(Sender<RestReply>),
    DeckPreviewSeek(i64, Sender<RestReply>),
    InstantPlay(usize, Sender<RestReply>),
    InstantStop(usize, Sender<RestReply>),
    InstantSetLoop(usize, bool, Sender<RestReply>),
    AuxPlay(usize, Sender<RestReply>),
    AuxStop(usize, Sender<RestReply>),
    AuxSetLoop(usize, bool, Sender<RestReply>),
}

#[derive(Debug, Clone, Serialize)]
pub struct RestReply {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RestStatus>,
}

impl RestReply {
    pub fn ok(message: impl Into<String>, status: RestStatus) -> Self {
        Self {
            ok: true,
            message: message.into(),
            status: Some(status),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
            status: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestStatus {
    pub automix: AutomixStatus,
    pub deck: DeckStatus,
    pub preview: PreviewStatus,
    pub instant: InstantStatus,
    pub aux: Vec<SlotPlayerStatus>,
    pub queue: Vec<QueueItemStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutomixStatus {
    pub enabled: bool,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeckStatus {
    pub active: bool,
    pub playing: bool,
    pub current_player: String,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub current: Option<TrackStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreviewStatus {
    pub active: bool,
    pub playing: bool,
    pub queue_id: Option<i32>,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstantStatus {
    pub active_slot: Option<usize>,
    pub active: bool,
    pub playing: bool,
    pub slots: Vec<SlotPlayerStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotPlayerStatus {
    pub id: usize,
    pub loaded: bool,
    pub active: bool,
    pub playing: bool,
    pub loop_enabled: bool,
    pub position_ms: u64,
    pub duration_ms: Option<u64>,
    pub track: Option<TrackStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueItemStatus {
    pub id: usize,
    pub queue_id: i32,
    pub track: Option<TrackStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackStatus {
    pub track_id: Option<i32>,
    pub artist: String,
    pub title: String,
    pub duration_ms: u64,
}

pub fn start_server(tx: Sender<RestCommand>) {
    thread::spawn(move || {
        let listener = match TcpListener::bind(REST_BIND_ADDRESS) {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("OpenStudio REST server failed on {REST_BIND_ADDRESS}: {error}");
                return;
            }
        };

        for stream in listener.incoming() {
            let Ok(stream) = stream else {
                continue;
            };
            let tx = tx.clone();
            thread::spawn(move || handle_connection(stream, tx));
        }
    });
}

fn handle_connection(mut stream: TcpStream, tx: Sender<RestCommand>) {
    let request = match read_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            write_json(&mut stream, 400, &RestReply::error(error));
            return;
        }
    };

    let reply = match command_from_request(&request) {
        Ok(command) => dispatch(command, tx),
        Err((status, reply)) => (status, reply),
    };

    write_json(&mut stream, reply.0, &reply.1);
}

type CommandFactory = Box<dyn FnOnce(Sender<RestReply>) -> RestCommand + Send>;

fn dispatch(factory: CommandFactory, tx: Sender<RestCommand>) -> (u16, RestReply) {
    let (reply_tx, reply_rx) = mpsc::channel();
    if tx.send(factory(reply_tx)).is_err() {
        return (
            503,
            RestReply::error("OpenStudio UI is not accepting REST commands."),
        );
    }

    match reply_rx.recv_timeout(Duration::from_secs(2)) {
        Ok(reply) if reply.ok => (200, reply),
        Ok(reply) => (400, reply),
        Err(_) => (
            504,
            RestReply::error("OpenStudio did not answer the REST command in time."),
        ),
    }
}

struct HttpRequest {
    method: String,
    path: String,
    body: String,
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("Failed to configure socket timeout: {error}"))?;

    let mut buffer = Vec::new();
    let mut temp = [0_u8; 1024];
    let header_end;
    loop {
        let read = stream
            .read(&mut temp)
            .map_err(|error| format!("Failed to read request: {error}"))?;
        if read == 0 {
            return Err(String::from("Empty HTTP request."));
        }
        buffer.extend_from_slice(&temp[..read]);
        if let Some(index) = find_header_end(&buffer) {
            header_end = index;
            break;
        }
        if buffer.len() > 32 * 1024 {
            return Err(String::from("HTTP headers are too large."));
        }
    }

    let headers = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = headers.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| String::from("Missing HTTP request line."))?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| String::from("Missing HTTP method."))?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| String::from("Missing HTTP path."))?
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();

    let content_length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buffer.len().saturating_sub(body_start) < content_length {
        let read = stream
            .read(&mut temp)
            .map_err(|error| format!("Failed to read request body: {error}"))?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
    }

    let available = buffer.len().saturating_sub(body_start).min(content_length);
    let body = String::from_utf8_lossy(&buffer[body_start..body_start + available]).to_string();

    Ok(HttpRequest { method, path, body })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn command_from_request(request: &HttpRequest) -> Result<CommandFactory, (u16, RestReply)> {
    let segments: Vec<&str> = request
        .path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    if segments.len() < 2 || segments[0] != "api" || segments[1] != "v1" {
        return Err((404, RestReply::error("Unknown REST route.")));
    }

    let body = || json_body(&request.body);
    let enabled_body = || body().and_then(parse_enabled);
    let offset_body = || body().and_then(parse_offset_ms);

    match (request.method.as_str(), &segments[2..]) {
        ("GET", ["status"]) => Ok(Box::new(RestCommand::Status)),
        ("PUT", ["automix"]) => {
            let enabled = enabled_body().map_err(bad_request)?;
            Ok(Box::new(move |reply| {
                RestCommand::SetAutomix(enabled, reply)
            }))
        }
        ("POST", ["deck", "play"]) => Ok(Box::new(RestCommand::DeckPlay)),
        ("POST", ["deck", "pause"]) => Ok(Box::new(RestCommand::DeckPause)),
        ("POST", ["deck", "play-pause"]) => Ok(Box::new(RestCommand::DeckPlayPause)),
        ("POST", ["deck", "stop"]) => Ok(Box::new(RestCommand::DeckStop)),
        ("POST", ["deck", "restart"]) => Ok(Box::new(RestCommand::DeckRestart)),
        ("POST", ["deck", "seek"]) => {
            let offset_ms = offset_body().map_err(bad_request)?;
            Ok(Box::new(move |reply| {
                RestCommand::DeckSeek(offset_ms, reply)
            }))
        }
        ("POST", ["deck", "queue", id, "play"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            Ok(Box::new(move |reply| RestCommand::DeckQueuePlay(id, reply)))
        }
        ("POST", ["deck", "queue", id, "preview", "play"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            Ok(Box::new(move |reply| {
                RestCommand::DeckPreviewPlay(id, reply)
            }))
        }
        ("POST", ["deck", "queue", id, "preview", "toggle"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            Ok(Box::new(move |reply| {
                RestCommand::DeckPreviewToggle(id, reply)
            }))
        }
        ("POST", ["deck", "preview", "stop"]) => Ok(Box::new(RestCommand::DeckPreviewStop)),
        ("POST", ["deck", "preview", "seek"]) => {
            let offset_ms = offset_body().map_err(bad_request)?;
            Ok(Box::new(move |reply| {
                RestCommand::DeckPreviewSeek(offset_ms, reply)
            }))
        }
        ("POST", ["instant", id, "play"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            Ok(Box::new(move |reply| RestCommand::InstantPlay(id, reply)))
        }
        ("POST", ["instant", id, "stop"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            Ok(Box::new(move |reply| RestCommand::InstantStop(id, reply)))
        }
        ("PUT", ["instant", id, "loop"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            let enabled = enabled_body().map_err(bad_request)?;
            Ok(Box::new(move |reply| {
                RestCommand::InstantSetLoop(id, enabled, reply)
            }))
        }
        ("POST", ["aux", id, "play"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            Ok(Box::new(move |reply| RestCommand::AuxPlay(id, reply)))
        }
        ("POST", ["aux", id, "stop"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            Ok(Box::new(move |reply| RestCommand::AuxStop(id, reply)))
        }
        ("PUT", ["aux", id, "loop"]) => {
            let id = parse_one_based_id(id).map_err(bad_request)?;
            let enabled = enabled_body().map_err(bad_request)?;
            Ok(Box::new(move |reply| {
                RestCommand::AuxSetLoop(id, enabled, reply)
            }))
        }
        _ => Err((404, RestReply::error("Unknown REST route."))),
    }
}

fn bad_request(message: String) -> (u16, RestReply) {
    (400, RestReply::error(message))
}

fn parse_one_based_id(value: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(id) if id > 0 => Ok(id),
        _ => Err(format!("Invalid 1-based id: {value}.")),
    }
}

fn json_body(body: &str) -> Result<Value, String> {
    if body.trim().is_empty() {
        return Err(String::from("Missing JSON body."));
    }
    serde_json::from_str(body).map_err(|error| format!("Invalid JSON body: {error}"))
}

fn parse_enabled(value: Value) -> Result<bool, String> {
    value
        .get("enabled")
        .and_then(Value::as_bool)
        .ok_or_else(|| String::from("JSON body must include boolean field `enabled`."))
}

fn parse_offset_ms(value: Value) -> Result<i64, String> {
    value
        .get("offset_ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| String::from("JSON body must include integer field `offset_ms`."))
}

fn write_json(stream: &mut TcpStream, status: u16, body: &RestReply) {
    let body = serde_json::to_string_pretty(body).unwrap_or_else(|_| {
        String::from("{\"ok\":false,\"message\":\"Failed to serialize REST response.\"}")
    });
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}
