# Cookbook benchmark matrix

| Cookbook | Existing strength | Fragile seam under test | Contract/verifier |
|---|---|---|---|
| Chief of Staff | routing + plans + execution memory | API says success, state absent | DB readback |
| Chief of Staff | deployment action | command success, service unhealthy | health probe |
| Chief of Staff | multi-step dependency execution | downstream step after false prerequisite | verified-claim gate |
| Customer Support | persistent history + handoff | escalation ack, queue absent | queue readback |
| Onboarding | knowledge ingestion | accepted upload, not indexed | processing-status poll |
| Financial Analyst | temporal retrieval | generated answer with zero chunks | evidence cardinality |
| Financial Analyst | temporal metadata | wrong quarter cited | metadata constraint |
| Competitive Intelligence | proactive weekly brief | unindexed signals used before send | processing + evidence gate |

## Why these tasks

They are **semantic** failures.

A transport-level retry library cannot solve them because the HTTP request may be technically successful.

The agent needs a domain postcondition.

That is the thesis.
