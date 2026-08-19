use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct LabState {
    db: Arc<Mutex<Connection>>,
}

impl LabState {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS customers(
                ns TEXT NOT NULL,
                id TEXT NOT NULL,
                email TEXT NOT NULL,
                PRIMARY KEY(ns,id)
            );
            CREATE TABLE IF NOT EXISTS welcome_messages(
                ns TEXT NOT NULL,
                customer_id TEXT NOT NULL,
                message_id TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS deployments(
                ns TEXT NOT NULL,
                service TEXT NOT NULL,
                healthy INTEGER NOT NULL,
                PRIMARY KEY(ns,service)
            );
            CREATE TABLE IF NOT EXISTS escalations(
                ns TEXT NOT NULL,
                ticket_id TEXT NOT NULL,
                PRIMARY KEY(ns,ticket_id)
            );
            CREATE TABLE IF NOT EXISTS uploads(
                ns TEXT NOT NULL,
                source_id TEXT NOT NULL,
                status TEXT NOT NULL,
                PRIMARY KEY(ns,source_id)
            );
            "#,
        )?;
        Ok(Self { db: Arc::new(Mutex::new(conn)) })
    }

    pub fn reset_namespace(&self, ns: &str) -> anyhow::Result<()> {
        let db = self.db.lock().unwrap();
        for table in ["customers", "welcome_messages", "deployments", "escalations", "uploads"] {
            db.execute(&format!("DELETE FROM {} WHERE ns=?1", table), params![ns])?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CrmCreate {
    pub ns: String,
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub fault: String,
}

async fn crm_create(
    State(st): State<LabState>,
    Json(x): Json<CrmCreate>,
) -> (StatusCode, Json<Value>) {
    let db = st.db.lock().unwrap();
    match x.fault.as_str() {
        "silent_noop" => {}
        "wrong_record" => {
            let wrong = format!("{}-wrong", x.id);
            let _ = db.execute(
                "INSERT OR REPLACE INTO customers(ns,id,email) VALUES (?1,?2,?3)",
                params![x.ns, wrong, x.email],
            );
        }
        _ => {
            let _ = db.execute(
                "INSERT OR REPLACE INTO customers(ns,id,email) VALUES (?1,?2,?3)",
                params![x.ns, x.id, x.email],
            );
        }
    }
    (
        StatusCode::OK,
        Json(json!({"success": true, "customer_id": x.id})),
    )
}

async fn crm_get(
    State(st): State<LabState>,
    AxumPath((ns, id)): AxumPath<(String, String)>,
) -> Json<Value> {
    let db = st.db.lock().unwrap();
    let exists: Option<i64> = db
        .query_row(
            "SELECT 1 FROM customers WHERE ns=?1 AND id=?2",
            params![ns, id],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    Json(json!({"exists": exists.is_some()}))
}

#[derive(Debug, Deserialize)]
pub struct Welcome {
    pub ns: String,
    pub customer_id: String,
    pub message_id: String,
}

async fn welcome(
    State(st): State<LabState>,
    Json(x): Json<Welcome>,
) -> Json<Value> {
    let db = st.db.lock().unwrap();
    let _ = db.execute(
        "INSERT INTO welcome_messages(ns,customer_id,message_id) VALUES (?1,?2,?3)",
        params![x.ns, x.customer_id, x.message_id],
    );
    Json(json!({"success": true, "message_id": x.message_id}))
}

async fn welcome_count(
    State(st): State<LabState>,
    AxumPath((ns, id)): AxumPath<(String, String)>,
) -> Json<Value> {
    let db = st.db.lock().unwrap();
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM welcome_messages WHERE ns=?1 AND customer_id=?2",
            params![ns, id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Json(json!({"count": count}))
}

#[derive(Debug, Deserialize)]
pub struct Deploy {
    pub ns: String,
    pub service: String,
    #[serde(default)]
    pub fault: String,
}

async fn deploy(
    State(st): State<LabState>,
    Json(x): Json<Deploy>,
) -> Json<Value> {
    let healthy = if x.fault == "false_green" { 0 } else { 1 };
    let db = st.db.lock().unwrap();
    let _ = db.execute(
        "INSERT OR REPLACE INTO deployments(ns,service,healthy) VALUES (?1,?2,?3)",
        params![x.ns, x.service, healthy],
    );
    Json(json!({"success": true, "deployment_id": format!("dep-{}", x.service)}))
}

async fn deploy_health(
    State(st): State<LabState>,
    AxumPath((ns, service)): AxumPath<(String, String)>,
) -> Json<Value> {
    let db = st.db.lock().unwrap();
    let healthy: Option<i64> = db
        .query_row(
            "SELECT healthy FROM deployments WHERE ns=?1 AND service=?2",
            params![ns, service],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    Json(json!({"exists": healthy.is_some(), "healthy": healthy == Some(1)}))
}

#[derive(Debug, Deserialize)]
pub struct Escalate {
    pub ns: String,
    pub ticket_id: String,
    #[serde(default)]
    pub fault: String,
}

async fn escalate(
    State(st): State<LabState>,
    Json(x): Json<Escalate>,
) -> Json<Value> {
    if x.fault != "silent_noop" {
        let db = st.db.lock().unwrap();
        let _ = db.execute(
            "INSERT OR REPLACE INTO escalations(ns,ticket_id) VALUES (?1,?2)",
            params![x.ns, x.ticket_id],
        );
    }
    Json(json!({"success": true, "queued": true, "ticket_id": x.ticket_id}))
}

async fn escalation_get(
    State(st): State<LabState>,
    AxumPath((ns, ticket)): AxumPath<(String, String)>,
) -> Json<Value> {
    let db = st.db.lock().unwrap();
    let exists: Option<i64> = db
        .query_row(
            "SELECT 1 FROM escalations WHERE ns=?1 AND ticket_id=?2",
            params![ns, ticket],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    Json(json!({"queued": exists.is_some()}))
}

#[derive(Debug, Deserialize)]
pub struct Upload {
    pub ns: String,
    pub source_id: String,
    #[serde(default)]
    pub fault: String,
}

async fn upload(
    State(st): State<LabState>,
    Json(x): Json<Upload>,
) -> Json<Value> {
    let status = if x.fault == "accepted_not_indexed" { "queued" } else { "indexed" };
    let db = st.db.lock().unwrap();
    let _ = db.execute(
        "INSERT OR REPLACE INTO uploads(ns,source_id,status) VALUES (?1,?2,?3)",
        params![x.ns, x.source_id, status],
    );
    Json(json!({"success": true, "accepted": true, "source_id": x.source_id}))
}

async fn upload_status(
    State(st): State<LabState>,
    AxumPath((ns, source_id)): AxumPath<(String, String)>,
) -> Json<Value> {
    let db = st.db.lock().unwrap();
    let status: Option<String> = db
        .query_row(
            "SELECT status FROM uploads WHERE ns=?1 AND source_id=?2",
            params![ns, source_id],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);
    Json(json!({"status": status.unwrap_or_else(|| "missing".to_string())}))
}

pub fn router(state: LabState) -> Router {
    Router::new()
        .route("/lab/crm/customers", post(crm_create))
        .route("/lab/crm/customers/:ns/:id", get(crm_get))
        .route("/lab/welcome", post(welcome))
        .route("/lab/welcome/:ns/:id", get(welcome_count))
        .route("/lab/deployments", post(deploy))
        .route("/lab/deployments/:ns/:service/health", get(deploy_health))
        .route("/lab/support/escalations", post(escalate))
        .route("/lab/support/escalations/:ns/:ticket", get(escalation_get))
        .route("/lab/ingestion/upload", post(upload))
        .route("/lab/ingestion/status/:ns/:source_id", get(upload_status))
        .with_state(state)
}

pub struct LabServer {
    pub base_url: String,
    pub state: LabState,
    _task: tokio::task::JoinHandle<()>,
}

impl LabServer {
    pub async fn spawn(db_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let state = LabState::open(db_path)?;
        let app = router(state.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr: SocketAddr = listener.local_addr()?;
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        Ok(Self {
            base_url: format!("http://{}", addr),
            state,
            _task: task,
        })
    }
}
