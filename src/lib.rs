use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use http_req::{
    request::{Method, Request},
    response::Response,
    uri::Uri,
};
use serde::{Deserialize, Serialize};
use std::env;
use webhook_flows::{handler, request_handler, send_response};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(
    _headers: Vec<(String, String)>,
    _subpath: String,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    logger::init();

    let mut router = Router::new();
    router
        .insert("/projects/:project_name", vec![get(projects_list)])
        .unwrap();
    router
        .insert("/issues/:project_name", vec![get(issues_list)])
        .unwrap();

    router
        .insert("/project/:project_name", vec![post(decline_project)])
        .unwrap();

    router
        .insert("/project/:project_name", vec![post(approve_budget)])
        .unwrap();

    router
        .insert("/project/:project_name", vec![post(final_approval)])
        .unwrap();

    if let Err(e) = route(router).await {
        match e {
            RouteError::NotFound => {
                send_response(404, vec![], b"No route matched".to_vec());
            }
            RouteError::MethodNotAllowed => {
                send_response(405, vec![], b"Method not allowed".to_vec());
            }
        }
    }
}

async fn track_and_redirect(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    let urls_map = create_map().await;

    match _qry.get("file_name") {
        Some(m) => match serde_json::from_value::<String>(m.clone()) {
            Ok(key) => match urls_map.contains_key(&key) {
                true => {
                    let download_url = match urls_map.get(&key) {
                        Some(u) => u,
                        None => {
                            log::error!("missing download_url for file: {}", key);
                            return;
                        }
                    };
                    let download_count = match store_flows::get(&key) {
                        Some(val) => match serde_json::from_value::<i32>(val) {
                            Ok(n) => n + 1,
                            Err(_e) => {
                                log::error!("failed to parse download_count from store: {}", _e);
                                1
                            }
                        },
                        None => 1,
                    };
                    store_flows::set(&key, serde_json::json!(download_count), None);

                    log::info!("{} downloaed {} times", key, download_count);

                    send_response(
                        302, // HTTP status code for Found (Redirection)
                        vec![
                            ("Location".to_string(), download_url.to_string()), // Redirect URL in the Location header
                        ],
                        Vec::new(), // No body for a redirection response
                    );
                }

                false => {
                    log::error!("invalid file_name: {}", key);
                    return;
                }
            },
            Err(_e) => {
                log::error!("failed to parse file_name: {}", _e);
                return;
            }
        },
        _ => {
            log::error!("missing file_name");
            return;
        }
    }
}
