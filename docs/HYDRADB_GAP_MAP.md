# HydraDB cookbook gap map

This document maps Iolaus to the **application architecture exposed in HydraDB's public cookbooks**.

It is not a vulnerability report against HydraDB's database internals.

## 1. Chief of Staff — function routing

HydraDB's cookbook architecture:

```text
natural-language task
      ↓
HydraDB function recall
      ↓
orchestrator
      ↓
policy
      ↓
real API
      ↓
result
      ↓
execution outcome stored as HydraDB memory
      ↓
future routing learns
```

HydraDB's published metrics cover:

- top-1 routing accuracy;
- multi-step plan completion;
- personalization;
- acceptance;
- routing/plan latency.

Iolaus adds:

- semantic postcondition verification;
- false trusted-success commit rate;
- downstream contamination;
- verified outcome learning signal;
- cost per verified success.

### Why this is complementary

Routing accuracy answers:

> Did we choose the intended function?

Iolaus asks:

> Did the chosen function actually make the intended state true?

These are orthogonal.

## 2. Customer Support Agent

The public cookbook:

1. recalls KB + customer memory;
2. generates an LLM reply;
3. stores the exchange back to HydraDB;
4. offers escalation/human handoff.

Fragile boundaries:

- a reply may claim an external remedy occurred when no side effect was performed;
- an escalation connector may acknowledge without creating a queue item;
- a false resolution can become persistent customer history.

Iolaus contract examples:

```text
credit_applied:
  verifier = billing-ledger readback

human_handoff:
  verifier = queue/ticket readback

issue_resolved:
  verifier = customer confirmation OR deterministic telemetry
```

## 3. Onboarding / ingestion

HydraDB's onboarding documentation explicitly separates **upload acceptance** from **index readiness** and recommends waiting / verifying processing before recall.

Iolaus generalises this pattern:

```text
upload accepted
≠
knowledge usable

verify_processing PASS
→ verified_indexed:{source_id}
```

Any downstream recall step can require that verified claim.

## 4. Financial Analyst

The public cookbook's own pitfalls include:

- wrong timestamps causing the wrong quarter to surface;
- empty chunks leading to confident hallucinations;
- mismatched sub-tenant reads producing empty results;
- querying before `verify_processing`.

Iolaus turns these from comments into executable contracts:

```text
answer_financial_question requires:
  evidence_count >= 1
  evidence.period matches requested period
  evidence source indexed
```

This is precondition/postcondition enforcement, not retrieval replacement.

## 5. Competitive Intelligence

The weekly briefing agent proactively sends Slack briefings from recalled signals.

Fragile boundary:

```text
scheduled autonomous output
        ↓
no human requester present to notice missing/empty/stale source state
```

Iolaus can require:

- source ingestion verified;
- minimum evidence cardinality;
- recency window satisfied;
- optional multi-source corroboration.

## 6. Internal IT / decision provenance

HydraDB excels at recovering decision context.

For high-confidence automated answers, Iolaus can add:

```text
answer_decision_question requires:
  >= 1 evidence chunk
  >= 1 provenance/query path
  cited source identifiers present
```

The key distinction:

HydraDB retrieves and ranks context.
Iolaus governs when a **derived action or assertion** may be promoted to trusted state.

## 7. Why this matters more as autonomy increases

The cookbook trajectory already includes:

- scheduled jobs;
- system events;
- multi-step plans;
- persistent execution memory;
- self-improvement;
- automatic Slack output.

Human correction becomes less available as these loops become more autonomous.

A false success can therefore propagate:

```text
false tool success
      ↓
positive execution memory
      ↓
future router preference
      ↓
downstream prerequisite treated as satisfied
      ↓
additional autonomous actions
```

Iolaus cuts this at the first boundary.
