use std::path::PathBuf;

use clap::{Parser, Subcommand};
use iolaus_bench::{
    certify_live, find_first_function_id, HermesConfig, HermesExtractor, LabServer, LiveRun, LiveRunner,
    LiveTaskSuite, Runner, SuiteFile,
};
use iolaus_hydra::{HydraDbClient, RecallRequest};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(name="iolaus-bench", about="Paired semantic-failure benchmark for verified agent state transitions")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Controlled paired benchmark against the local HTTP+SQLite target lab.
    Run {
        #[arg(long, default_value="benchmarks/hydradb-cookbooks.toml")]
        suite: PathBuf,
        #[arg(long)]
        trials: Option<u64>,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long, default_value="results/run.json")]
        out: PathBuf,
        #[arg(long)]
        hydra_feedback: bool,
    },
    /// Recompute controlled benchmark metrics from raw trials and verify receipts.
    Certify { path: PathBuf },
    /// Live HydraDB + local Hermes decision-plane benchmark plus paired Iolaus execution.
    LiveRun {
        #[arg(long, default_value="benchmarks/hydradb-cookbooks.toml")]
        suite: PathBuf,
        #[arg(long, default_value="fixtures/tasks/hydradb-live-tasks.json")]
        tasks: PathBuf,
        #[arg(long, default_value="fixtures/hydradb/functions.json")]
        functions: PathBuf,
        #[arg(long, default_value_t=5)]
        trials: u64,
        #[arg(long, default_value_t=20260819)]
        seed: u64,
        #[arg(long, default_value="results/live/live.json")]
        out: PathBuf,
        #[arg(long)]
        hydra_feedback: bool,
    },
    /// Recompute live metrics and verify anti-cheat invariants/receipts.
    LiveCertify { path: PathBuf },
    HydraSmoke,
    HermesSmoke,
    BootstrapHydra {
        #[arg(long, default_value="fixtures/hydradb/functions.json")]
        functions: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli=Cli::parse();
    match cli.command {
        Command::Run{suite,trials,seed,out,hydra_feedback}=>{
            let suite_file=SuiteFile::load(&suite)?;
            let n=trials.unwrap_or(suite_file.trials_per_scenario);
            let seed=seed.unwrap_or(suite_file.seed);
            let temp=std::env::temp_dir().join(format!("iolaus-lab-{}.sqlite",uuid::Uuid::new_v4()));
            let lab=LabServer::spawn(&temp).await?;
            let hydra=if hydra_feedback{Some(HydraDbClient::from_env()?)}else{None};
            let runner=Runner::new(lab.base_url.clone(),hydra);
            let run=runner.run_suite(&suite_file,n,seed).await?;
            if let Some(parent)=out.parent(){std::fs::create_dir_all(parent)?;}
            std::fs::write(&out,serde_json::to_vec_pretty(&run)?)?;
            println!("{}",serde_json::to_string_pretty(&run.summary)?);
            println!("raw results: {}",out.display());
        }
        Command::Certify{path}=>{
            let bytes=std::fs::read(&path)?;
            let run:iolaus_bench::runner::BenchmarkRun=serde_json::from_slice(&bytes)?;
            let recomputed=iolaus_core::summarize(&run.trials);
            anyhow::ensure!(serde_json::to_value(&run.summary)?==serde_json::to_value(&recomputed)?,"stored summary does not match raw trials");
            for t in &run.trials{
                anyhow::ensure!(t.baseline.fault_injected==t.verified.fault_injected,"paired arms did not receive the same fault assignment");
                if let Some(r)=&t.verified.receipt{
                    iolaus_core::ReceiptSigner::verify(r)?;
                    anyhow::ensure!(t.verified.trusted_success,"receipt attached to non-trusted transition");
                    anyhow::ensure!(t.verified.postcondition_true,"receipt attached to false postcondition");
                }
            }
            println!("CERTIFIED: raw trials reproduce aggregate metrics; pair assignment and all receipts verify");
        }
        Command::LiveRun{suite,tasks,functions,trials,seed,out,hydra_feedback}=>{
            let controlled=SuiteFile::load(&suite)?;
            let live_tasks=LiveTaskSuite::load(&tasks)?;
            let function_values:Vec<Value>=serde_json::from_slice(&std::fs::read(&functions)?)?;
            let hydra=HydraDbClient::from_env()?;
            let hermes=HermesExtractor::new(HermesConfig::default());
            let temp=std::env::temp_dir().join(format!("iolaus-live-lab-{}.sqlite",uuid::Uuid::new_v4()));
            let lab=LabServer::spawn(&temp).await?;
            let runner=Runner::new(lab.base_url.clone(),None);
            let live=LiveRunner::new(runner,hydra,hermes,function_values)
                .run(&controlled,&live_tasks,trials,seed,hydra_feedback).await?;
            if let Some(parent)=out.parent(){std::fs::create_dir_all(parent)?;}
            std::fs::write(&out,serde_json::to_vec_pretty(&live)?)?;
            println!("{}",serde_json::to_string_pretty(&live.summary)?);
            println!("live raw results: {}",out.display());
        }
        Command::LiveCertify{path}=>{
            let run:LiveRun=serde_json::from_slice(&std::fs::read(&path)?)?;
            certify_live(&run)?;
            println!("LIVE CERTIFIED: raw trials reproduce metrics; no fault leakage; pair assignment and receipts verify; verified learning signal has zero false-positive promotions");
        }
        Command::HydraSmoke=>{
            let hydra=HydraDbClient::from_env()?;
            let create=hydra.create_tenant_if_supported().await;
            match create { Ok(v)=>println!("tenant/create: {}",serde_json::to_string(&v)?), Err(e)=>eprintln!("tenant/create not available or failed (continuing): {e}") }
            let nonce=uuid::Uuid::new_v4().to_string();
            let memory=json!({"event":"iolaus_smoke","ok":true,"nonce":nonce.clone()}).to_string();
            hydra.add_memory("iolaus-smoke",&memory,"iolaus-benchmark",false).await?;
            let mut last=Value::Null;
            let mut observed=false;
            for _ in 0..12 {
                let recall=hydra.recall(RecallRequest{
                    query:format!("iolaus smoke benchmark event nonce {nonce}"),
                    sub_tenant_id:Some("iolaus-smoke".into()),max_results:5,mode:"fast".into(),graph_context:false
                }).await?;
                observed=serde_json::to_string(&recall)?.contains(&nonce);
                last=recall;
                if observed { break; }
                tokio::time::sleep(std::time::Duration::from_millis(750)).await;
            }
            println!("recall response: {}",serde_json::to_string_pretty(&last)?);
            anyhow::ensure!(observed,"HydraDB write was not observable by recall after retries");
            println!("HYDRA_SMOKE_PASS");
        }
        Command::HermesSmoke=>{
            let h=HermesExtractor::new(HermesConfig::default());
            let v=h.smoke().await?;
            anyhow::ensure!(v.get("ok").and_then(Value::as_bool)==Some(true),"Hermes smoke JSON did not contain ok=true");
            println!("{}",serde_json::to_string_pretty(&v)?);
            println!("HERMES_SMOKE_PASS");
        }
        Command::BootstrapHydra{functions}=>{
            let hydra=HydraDbClient::from_env()?;
            let _=hydra.create_tenant_if_supported().await;
            let schemas:Value=serde_json::from_slice(&std::fs::read(functions)?)?;
            let response=hydra.upload_app_knowledge("iolaus-functions",&schemas).await?;
            println!("functions upload: {}",serde_json::to_string_pretty(&response)?);
            for (sub,text,user,infer) in [
                ("iolaus-preferences","Sarah prefers Slack for urgent internal updates.","sarah",true),
                ("iolaus-policies","A deployment is complete only after an independent health readback.","system",true),
                ("iolaus-policies","An accepted knowledge upload is not recall-ready until processing is indexed.","system",true),
                ("iolaus-policies","A human handoff is complete only when the ticket exists in the handoff queue.","system",true),
                ("iolaus-policies","Financial answers require non-empty evidence from the requested fiscal period.","system",true),
            ] {
                hydra.add_memory(sub,text,user,infer).await?;
            }
            let known:Vec<String>=schemas.as_array().into_iter().flatten()
                .filter_map(|v|v.get("id").and_then(Value::as_str).map(str::to_string)).collect();
            let mut last=Value::Null;
            let mut routed=None;
            for _ in 0..16 {
                let recall=hydra.recall(RecallRequest{
                    query:"deploy the api service".into(),sub_tenant_id:Some("iolaus-functions".into()),
                    max_results:5,mode:"thinking".into(),graph_context:true,
                }).await?;
                routed=find_first_function_id(&recall,&known);
                last=recall;
                if routed.as_deref()==Some("trigger_deployment") { break; }
                tokio::time::sleep(std::time::Duration::from_millis(750)).await;
            }
            println!("bootstrap recall probe: {}",serde_json::to_string_pretty(&last)?);
            anyhow::ensure!(routed.as_deref()==Some("trigger_deployment"),"function registry not recallable/routable after retries; selected={routed:?}");
            println!("BOOTSTRAP_HYDRA_PASS");
        }
    }
    Ok(())
}
