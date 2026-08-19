#!/usr/bin/env python3
"""Generate deterministic HydraDB benchmark fixtures.

This file deliberately contains no benchmark outcome labels or fault assignments.
The graph is context/routing state only. Fault assignment is performed inside the
Rust runner from (seed, scenario_id, trial_index), after routing/extraction.
"""
from __future__ import annotations
import argparse, hashlib, json
from pathlib import Path

FUNCTIONS = [
    {
        "id":"create_customer","name":"Create CRM customer",
        "description":"Create a new CRM customer record. Use when a user asks to add, create, or register a customer/contact in CRM. Semantic success requires independent readback of the requested customer ID.",
        "parameters":{"type":"object","properties":{"id":{"type":"string"},"email":{"type":"string"}},"required":["id","email"]},
        "meta":{"department":"sales","collections":["crm","write"],"idempotent":True,"side_effects":"Writes CRM state","verification":"crm_readback"}
    },
    {
        "id":"update_crm_opportunity","name":"Update CRM opportunity",
        "description":"Update stage, amount, owner, or notes on an existing CRM opportunity.",
        "parameters":{"type":"object","properties":{"opportunity_id":{"type":"string"},"stage":{"type":"string"}},"required":["opportunity_id","stage"]},
        "meta":{"department":"sales","collections":["crm","write"],"idempotent":True}
    },
    {
        "id":"send_slack_message","name":"Send Slack message",
        "description":"Post an internal message to a Slack channel or direct message. Prefer for time-sensitive internal communication.",
        "parameters":{"type":"object","properties":{"channel":{"type":"string"},"text":{"type":"string"}},"required":["channel","text"]},
        "meta":{"department":"all","collections":["communication"],"idempotent":False}
    },
    {
        "id":"send_email","name":"Send email",
        "description":"Send an external or formal email message to one or more recipients.",
        "parameters":{"type":"object","properties":{"to":{"type":"string"},"subject":{"type":"string"},"body":{"type":"string"}},"required":["to","subject","body"]},
        "meta":{"department":"all","collections":["communication"],"idempotent":False}
    },
    {
        "id":"create_calendar_event","name":"Create calendar event",
        "description":"Create a calendar meeting or appointment with date, time and attendees.",
        "parameters":{"type":"object","properties":{"title":{"type":"string"},"start":{"type":"string"}},"required":["title","start"]},
        "meta":{"department":"all","collections":["calendar"],"idempotent":False}
    },
    {
        "id":"trigger_deployment","name":"Trigger deployment",
        "description":"Deploy a named service or application. A transport-level success is not sufficient; semantic success requires an independent health readback.",
        "parameters":{"type":"object","properties":{"service":{"type":"string"}},"required":["service"]},
        "meta":{"department":"engineering","collections":["deploy","write"],"idempotent":False,"verification":"health_probe"}
    },
    {
        "id":"notify_pagerduty","name":"Notify PagerDuty",
        "description":"Create or trigger an incident in PagerDuty for an operational alert.",
        "parameters":{"type":"object","properties":{"service":{"type":"string"},"summary":{"type":"string"}},"required":["service","summary"]},
        "meta":{"department":"engineering","collections":["incident"],"idempotent":False}
    },
    {
        "id":"create_jira_ticket","name":"Create Jira ticket",
        "description":"Create a Jira issue for work tracking, bug reports, or follow-up tasks.",
        "parameters":{"type":"object","properties":{"project":{"type":"string"},"summary":{"type":"string"}},"required":["project","summary"]},
        "meta":{"department":"all","collections":["project-management"],"idempotent":False}
    },
    {
        "id":"escalate_to_human","name":"Escalate support ticket",
        "description":"Queue a customer-support ticket for human handoff. Semantic success requires the handoff to exist in the queue.",
        "parameters":{"type":"object","properties":{"ticket_id":{"type":"string"}},"required":["ticket_id"]},
        "meta":{"department":"support","collections":["support","write"],"idempotent":True,"verification":"queue_readback"}
    },
    {
        "id":"upload_knowledge","name":"Upload knowledge source",
        "description":"Upload a document or source for indexing. Accepted upload does not imply recall-ready; downstream use requires processing/indexing verification.",
        "parameters":{"type":"object","properties":{"source_id":{"type":"string"}},"required":["source_id"]},
        "meta":{"department":"all","collections":["knowledge","ingestion"],"idempotent":True,"verification":"processing_ready"}
    },
    {
        "id":"answer_financial_question","name":"Answer financial question",
        "description":"Answer a financial question from retrieved evidence. Output is trusted only when evidence is non-empty and matches the requested fiscal period.",
        "parameters":{"type":"object","properties":{"question":{"type":"string"},"period":{"type":"string"}},"required":["question","period"]},
        "meta":{"department":"finance","collections":["analysis"],"idempotent":True,"verification":"evidence_constraints"}
    },
    {
        "id":"generate_competitive_briefing","name":"Generate competitive briefing",
        "description":"Generate a competitive-intelligence briefing from indexed source material. Do not publish until all required sources are processing-ready.",
        "parameters":{"type":"object","properties":{"source_id":{"type":"string"},"audience":{"type":"string"}},"required":["source_id","audience"]},
        "meta":{"department":"strategy","collections":["competitive-intelligence"],"idempotent":True,"verification":"source_readiness"}
    },
    {
        "id":"generate_report","name":"Generate report",
        "description":"Generate a general structured internal report from available context.",
        "parameters":{"type":"object","properties":{"topic":{"type":"string"}},"required":["topic"]},
        "meta":{"department":"all","collections":["reporting"],"idempotent":True}
    },
]

CUSTOMERS = [
    ("cust-alice","Alice Nguyen","alice@example.test","enterprise"),
    ("cust-ben","Ben Ortiz","ben@example.test","pro"),
    ("cust-cara","Cara Singh","cara@example.test","starter"),
    ("cust-dan","Dan Kato","dan@example.test","enterprise"),
    ("cust-eva","Eva Rossi","eva@example.test","pro"),
    ("cust-finn","Finn Adeyemi","finn@example.test","starter"),
]

MEMORIES = [
    ("mem-sarah-slack","sarah","Sarah prefers Slack for urgent internal updates.","preference"),
    ("mem-sarah-deploy","sarah","For production deployments, verify service health before announcing completion.","policy"),
    ("mem-support-handoff","support","Human escalation is only complete when the ticket is present in the handoff queue.","policy"),
    ("mem-ingest-ready","research","Uploaded knowledge is not usable until processing reports indexed/ready.","policy"),
    ("mem-finance-period","finance","Financial answers must cite evidence from the requested fiscal period.","policy"),
    ("mem-finance-evidence","finance","Do not treat an answer with zero evidence chunks as established.","policy"),
]

EXECUTIONS = [
    ("exec-001","sarah","send_slack_message","success",180),
    ("exec-002","sarah","send_email","user_rejected",310),
    ("exec-003","sarah","trigger_deployment","success",1240),
    ("exec-004","sarah","trigger_deployment","slow",3100),
    ("exec-005","support","escalate_to_human","success",420),
    ("exec-006","support","escalate_to_human","failure",530),
    ("exec-007","research","upload_knowledge","success",890),
    ("exec-008","research","generate_competitive_briefing","success",760),
    ("exec-009","finance","answer_financial_question","success",640),
    ("exec-010","finance","answer_financial_question","failure",610),
    ("exec-011","sales","create_customer","success",205),
    ("exec-012","sales","update_crm_opportunity","success",190),
]

TASKS = [
    {
        "id":"chief.crm_silent_write","kind":"crm_silent_write","expected_function":"create_customer",
        "tasks":["Add Alice to the CRM with id alice and email alice@example.test.","Create CRM customer alice using alice@example.test."],
        "expected_params":{"id":"alice","email":"alice@example.test"}
    },
    {
        "id":"chief.deploy_false_green","kind":"deploy_false_green","expected_function":"trigger_deployment",
        "tasks":["Deploy the api service.","Trigger a deployment for service api."],
        "expected_params":{"service":"api"}
    },
    {
        "id":"chief.cascade_welcome","kind":"cascade_welcome","expected_function":"create_customer",
        "tasks":["Create customer alice at alice@example.test, then welcome them only after creation is confirmed.","Register alice in CRM using alice@example.test before the welcome step."],
        "expected_params":{"id":"alice","email":"alice@example.test"}
    },
    {
        "id":"support.false_handoff","kind":"support_false_handoff","expected_function":"escalate_to_human",
        "tasks":["Escalate support ticket TCK-42 to a human.","Queue ticket TCK-42 for human support."],
        "expected_params":{"ticket_id":"TCK-42"}
    },
    {
        "id":"onboarding.accepted_not_indexed","kind":"accepted_not_indexed","expected_function":"upload_knowledge",
        "tasks":["Upload onboarding source source-1 for indexing.","Add source-1 to the onboarding knowledge base."],
        "expected_params":{"source_id":"source-1"}
    },
    {
        "id":"finance.empty_evidence","kind":"empty_evidence","expected_function":"answer_financial_question",
        "tasks":["What was gross margin in Q4 2025?", "Answer the gross-margin question for Q4-2025."],
        "expected_params":{"question":"What was gross margin in Q4 2025?","period":"Q4-2025"}
    },
    {
        "id":"finance.wrong_period","kind":"wrong_period","expected_function":"answer_financial_question",
        "tasks":["Use Q4 2025 evidence to answer the gross-margin question.","Report gross margin for Q4-2025 only."],
        "expected_params":{"question":"What was gross margin in Q4 2025?","period":"Q4-2025"}
    },
    {
        "id":"intel.unverified_briefing","kind":"unverified_briefing","expected_function":"generate_competitive_briefing",
        "tasks":["Prepare an executive competitor briefing from source-1 after the source is ready.","Generate the executive competitive-intelligence brief from source-1."],
        "expected_params":{"source_id":"source-1","audience":"executive"}
    },
]

def dump_jsonl(path: Path, rows):
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, sort_keys=True, separators=(",",":")) + "\n")

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--out", default="fixtures/hydradb")
    args=ap.parse_args()
    out=Path(args.out); out.mkdir(parents=True, exist_ok=True)
    (out/"functions.json").write_text(json.dumps(FUNCTIONS,indent=2,sort_keys=True)+"\n")
    (out.parent/"tasks"/"hydradb-live-tasks.json").parent.mkdir(parents=True,exist_ok=True)
    (out.parent/"tasks"/"hydradb-live-tasks.json").write_text(json.dumps({"schema":"iolaus.hydradb.live.tasks.v1","tasks":TASKS},indent=2,sort_keys=True)+"\n")

    nodes=[]; edges=[]
    for fn in FUNCTIONS:
        nodes.append({"id":f"fn:{fn['id']}","labels":["Function"],"properties":{"function_id":fn["id"],"name":fn["name"],"description":fn["description"],"parameters":fn["parameters"],"meta":fn["meta"]}})
    for cid,name,email,plan in CUSTOMERS:
        nodes.append({"id":cid,"labels":["Customer"],"properties":{"name":name,"email":email,"plan":plan}})
    for mid,user,text,kind in MEMORIES:
        nodes.append({"id":mid,"labels":["Memory"],"properties":{"user":user,"text":text,"memory_kind":kind}})
    for eid,user,fn,outcome,latency in EXECUTIONS:
        nodes.append({"id":eid,"labels":["Execution"],"properties":{"user":user,"function_id":fn,"outcome":outcome,"latency_ms":latency}})
        edges.append({"id":f"edge:{eid}:fn","labels":["EXECUTED_FUNCTION"],"from":eid,"to":f"fn:{fn}","properties":{"outcome":outcome,"latency_ms":latency}})
    for mid,user,_,kind in MEMORIES:
        if kind=="preference":
            edges.append({"id":f"edge:{mid}:slack","labels":["PREFERS"],"from":mid,"to":"fn:send_slack_message","properties":{"user":user}})
    for fn in FUNCTIONS:
        ver=fn.get("meta",{}).get("verification")
        if ver:
            verifier_id=f"verifier:{ver}"
            if not any(n["id"]==verifier_id for n in nodes):
                nodes.append({"id":verifier_id,"labels":["Verifier"],"properties":{"verifier_id":ver,"deterministic":True}})
            edges.append({"id":f"edge:verify:{fn['id']}","labels":["REQUIRES_VERIFICATION"],"from":f"fn:{fn['id']}","to":verifier_id,"properties":{}})

    dump_jsonl(out/"nodes.jsonl",nodes)
    dump_jsonl(out/"edges.jsonl",edges)
    manifest={
        "schema":"iolaus.hydradb.fixture.v1",
        "nodes":len(nodes),"edges":len(edges),"functions":len(FUNCTIONS),"customers":len(CUSTOMERS),"memories":len(MEMORIES),"executions":len(EXECUTIONS),
        "note":"Portable Iolaus fixture. The source-build adapter must map labels/properties to the exact HydraDB import schema discovered in the checked-out HydraDB revision; do not guess field names.",
    }
    for name in ["nodes.jsonl","edges.jsonl","functions.json"]:
        b=(out/name).read_bytes(); manifest[name+"_sha256"]=hashlib.sha256(b).hexdigest()
    (out/"manifest.json").write_text(json.dumps(manifest,indent=2,sort_keys=True)+"\n")
    print(json.dumps(manifest,indent=2))

if __name__=="__main__": main()
