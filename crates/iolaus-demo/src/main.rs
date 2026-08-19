use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use iolaus_bench::{LabServer, Runner, ScenarioSpec, SuiteFile};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t=8080)]
    port:u16,
}

#[derive(Clone)]
struct AppState{
    runner:Runner,
    suite:Arc<SuiteFile>,
}

#[tokio::main]
async fn main()->anyhow::Result<()>{
    let args=Args::parse();
    let db=std::env::temp_dir().join(format!("iolaus-demo-{}.sqlite",uuid::Uuid::new_v4()));
    let lab=LabServer::spawn(&db).await?;
    let suite=SuiteFile::load("benchmarks/hydradb-cookbooks.toml")?;
    let state=AppState{runner:Runner::new(lab.base_url.clone(),None),suite:Arc::new(suite)};
    let app=Router::new()
        .route("/",get(index))
        .route("/api/scenarios",get(scenarios))
        .route("/api/run/:id",post(run_one))
        .route("/api/suite",post(run_suite))
        .with_state(state)
        .layer(CorsLayer::permissive());
    let listener=tokio::net::TcpListener::bind(("127.0.0.1",args.port)).await?;
    println!("Iolaus demo: http://127.0.0.1:{}",args.port);
    axum::serve(listener,app).await?;
    Ok(())
}

async fn index()->Html<&'static str>{Html(include_str!("../static/index.html"))}

async fn scenarios(State(st):State<AppState>)->Json<Value>{
    Json(json!(st.suite.scenario))
}

async fn run_one(
    State(st):State<AppState>,
    Path(id):Path<String>,
)->Result<Json<Value>,(axum::http::StatusCode,String)>{
    let spec:ScenarioSpec=st.suite.scenario.iter().find(|s|s.id==id).cloned()
        .ok_or((axum::http::StatusCode::NOT_FOUND,"unknown scenario".into()))?;
    let seed=20260819;
    // Single-scenario demo deliberately forces the declared semantic fault.
    // The statistical suite below uses preregistered seeded probabilities.
    let trial=st.runner.run_paired(&spec,0,seed,true).await
        .map_err(internal)?;
    Ok(Json(json!(trial)))
}

async fn run_suite(State(st):State<AppState>)->Result<Json<Value>,(axum::http::StatusCode,String)>{
    // 25 per each of 8 scenarios = 200 paired trials total for a fast demo.
    let run=st.runner.run_suite(&st.suite,25,20260819).await.map_err(internal)?;
    Ok(Json(json!({"summary":run.summary,"trials":run.trials.len()})))
}

fn internal(e:anyhow::Error)->(axum::http::StatusCode,String){
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR,e.to_string())
}
