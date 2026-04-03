use std::sync::mpsc;

use anyhow::{Context, Result};

use super::diff::{get_all_file_contents, run_diff};
use super::types::{DiffData, FeedbackPayload, SwitchRequest};

const HTML: &str = include_str!("../../../../orca-review/dist/index.html");
const BASE_PORT: u16 = 19400;
const MAX_RETRIES: u16 = 5;

pub struct ReviewServer {
    pub url: String,
    rx: mpsc::Receiver<FeedbackPayload>,
    _handle: std::thread::JoinHandle<()>,
}

fn build_diff_data(
    patch: &str,
    git_ref: &str,
    diff_type: &str,
    default_branch: &str,
    error: &Option<String>,
) -> DiffData {
    let files = get_all_file_contents(patch, diff_type, default_branch);
    DiffData {
        raw_patch: patch.to_string(),
        git_ref: git_ref.to_string(),
        diff_type: diff_type.to_string(),
        default_branch: default_branch.to_string(),
        files,
        error: error.clone(),
    }
}

impl ReviewServer {
    pub fn start(
        initial_patch: String,
        initial_ref: String,
        initial_error: Option<String>,
        default_branch: String,
    ) -> Result<Self> {
        let server = {
            let mut server = None;
            for attempt in 0..MAX_RETRIES {
                match tiny_http::Server::http(format!("127.0.0.1:{}", BASE_PORT + attempt)) {
                    Ok(s) => {
                        server = Some(s);
                        break;
                    }
                    Err(_) if attempt < MAX_RETRIES - 1 => continue,
                    Err(e) => anyhow::bail!("failed to start server: {e}"),
                }
            }
            server.context("failed to start server")?
        };

        let port = server.server_addr().to_ip().unwrap().port();
        let url = format!("http://127.0.0.1:{port}");
        let (tx, rx) = mpsc::channel::<FeedbackPayload>();

        let handle = std::thread::spawn(move || {
            Self::serve(server, tx, initial_patch, initial_ref, initial_error, default_branch);
        });

        Ok(Self {
            url,
            rx,
            _handle: handle,
        })
    }

    pub fn wait_for_feedback(&self) -> Result<FeedbackPayload> {
        self.rx.recv().context("review server closed without feedback")
    }

    fn serve(
        server: tiny_http::Server,
        tx: mpsc::Sender<FeedbackPayload>,
        initial_patch: String,
        initial_ref: String,
        initial_error: Option<String>,
        default_branch: String,
    ) {
        let mut current_data = build_diff_data(
            &initial_patch, &initial_ref, "uncommitted", &default_branch, &initial_error,
        );

        for mut request in server.incoming_requests() {
            let path = request.url().to_string();
            let method = request.method().to_string();

            match (method.as_str(), path.as_str()) {
                ("GET", "/api/diff") => {
                    let _ = json_response(request, &serde_json::to_string(&current_data).unwrap());
                }
                ("POST", "/api/diff/switch") => {
                    let body = read_body(&mut request);
                    match serde_json::from_str::<SwitchRequest>(&body) {
                        Ok(req) if req.diff_type == "uncommitted" || req.diff_type == "branch" => {
                            let (patch, git_ref, error) =
                                run_diff(&req.diff_type, &default_branch);
                            current_data = build_diff_data(
                                &patch, &git_ref, &req.diff_type, &default_branch, &error,
                            );
                            let _ =
                                json_response(request, &serde_json::to_string(&current_data).unwrap());
                        }
                        _ => {
                            let _ = json_response(request, r#"{"error":"invalid diffType"}"#);
                        }
                    }
                }
                ("POST", "/api/feedback") => {
                    let body = read_body(&mut request);
                    match serde_json::from_str::<FeedbackPayload>(&body) {
                        Ok(payload) => {
                            let _ = json_response(request, r#"{"ok":true}"#);
                            let _ = tx.send(payload);
                            return;
                        }
                        Err(_) => {
                            let _ = json_response(request, r#"{"error":"invalid payload"}"#);
                        }
                    }
                }
                _ => {
                    let _ = html_response(request);
                }
            }
        }
    }
}

fn json_response(
    request: tiny_http::Request,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = tiny_http::Response::from_string(body).with_header(
        tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
    );
    request.respond(response)?;
    Ok(())
}

fn html_response(
    request: tiny_http::Request,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = tiny_http::Response::from_string(HTML).with_header(
        tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap(),
    );
    request.respond(response)?;
    Ok(())
}

fn read_body(request: &mut tiny_http::Request) -> String {
    let mut body = String::new();
    let _ = request.as_reader().read_to_string(&mut body);
    body
}
