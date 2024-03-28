pub mod db_updater;
use db_updater::*;
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use webhook_flows::{
    create_endpoint, request_handler,
    route::{get, post, route, RouteError, Router},
    send_response,
};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler(get, post)]
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
        .insert("/issues/:issue_id", vec![post(approve_issue_budget)])
        .unwrap();
    router
        .insert("/issues", vec![get(list_issues_handler)])
        .unwrap();

    // router
    //     .insert("/project/:project_name", vec![post(decline_project)])
    //     .unwrap();

    // router
    //     .insert("/project/:project_name", vec![post(approve_budget)])
    //     .unwrap();

    // router
    //     .insert("/project/:project_name", vec![post(final_approval)])
    //     .unwrap();

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

async fn approve_issue_budget(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    let issue_id = match _qry.get("issue_id") {
        Some(m) => match serde_json::from_value::<String>(m.clone()) {
            Ok(key) => key,
            Err(_e) => {
                log::error!("failed to parse issue_id: {}", _e);
                return;
            }
        },
        _ => {
            log::error!("missing issue_id");
            return;
        }
    };

    let issue_budget = match _qry.get("issue_budget") {
        Some(m) => match serde_json::from_value::<String>(m.clone()) {
            Ok(key) => key.parse::<i32>().unwrap_or_default(),
            Err(_e) => {
                log::error!("failed to parse issue_budget: {}", _e);
                return;
            }
        },
        _ => {
            log::error!("missing issue_budget");
            return;
        }
    };

    let pool = get_pool().await;

    // let _ = approve_issue_budget_in_db(&pool, &issue_id, issue_budget).await;
}
async fn list_issues_handler(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    let page = match _qry.get("page").and_then(|v| v.as_u64()) {
        Some(m) if m > 0 => m as usize,
        _ => {
            log::error!("Invalid or missing 'page' parameter");
            return;
        }
    };

    let page_size = match _qry.get("page_size").and_then(|v| v.as_u64()) {
        Some(m) if m > 0 => m as usize,
        _ => {
            log::error!("Invalid or missing 'page_size' parameter");
            return;
        }
    };
    log::error!("page: {}, page_size: {}", page, page_size);
    let pool = get_pool().await;

    let issues_obj = list_issues(&pool, page, page_size).await.expect("msg");

    let issues_str = format!("{:?}", issues_obj);
    log::error!("issues_str: {}", issues_str);

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        issues_str.as_bytes().to_vec(),
    );
}

async fn projects_list(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    // match _qry.get("file_name") {
    //     Some(m) => match serde_json::from_value::<String>(m.clone()) {
    //         Ok(key) => "file_name".to_string(),
    //         Err(_e) => {
    //             log::error!("failed to parse file_name: {}", _e);
    //             return;
    //         }
    //     },
    //     _ => {
    //         log::error!("missing file_name");
    //         return;
    //     }
    // }
}
