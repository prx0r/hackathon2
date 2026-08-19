# HydraDB evidence register

Checked 2026-08-19 against HydraDB public documentation.

## Official sources used

### Cookbooks index
https://docs.hydradb.com/cookbooks/v2

### Chief of Staff — Function Routing
https://docs.hydradb.com/cookbooks/hydradb-cookbook-06

Relevant public statements:
- functions are registered as knowledge objects;
- the orchestrator executes the selected function;
- execution outcomes are stored back into HydraDB;
- outcome memories become future self-improvement signal;
- benchmark: 3,200 task executions, 48 functions, 6 user profiles;
- reported metrics include routing accuracy, plan completion, personalization, acceptance, latency.

### Customer Support
https://docs.hydradb.com/cookbooks/customer-support-agent

Relevant public statements:
- two-call recall (KB + customer memory);
- every conversation turn is stored back;
- unresolved issues can be escalated to a human;
- benchmark covers 2,400 support tickets.

### Onboarding
https://docs.hydradb.com/cookbooks/ai-onboarding-agent

Relevant public statements:
- upload and indexing are distinct;
- no chunks may mean indexing is still ongoing;
- production notes recommend waiting before query.

### Financial Analyst
https://docs.hydradb.com/cookbooks/cookbook-10-ai-financial-analyst

Relevant public pitfalls:
- wrong timestamp → wrong period surfaces;
- no verify_processing → empty results;
- empty chunks passed to LLM → confident hallucinations;
- mismatched sub-tenant → empty results.

### Competitive Intelligence
https://docs.hydradb.com/cookbooks/competitive-intelligence-agent

Relevant public statements:
- source ingestion should be verified;
- weekly autonomous Slack briefing is generated from recall;
- benchmark covers temporal recall and stale-signal surface rate.

### Research paper
https://research.hydradb.com/hydradb.pdf

Relevant public claims:
- HydraDB models versioned, relational, time-aware state;
- LongMemEval-s evaluation reports 90.79% overall accuracy;
- temporal reasoning and knowledge updates are major evaluation categories.

## What Iolaus does NOT claim

- We do not claim HydraDB's database internally commits false state.
- We do not claim the published HydraDB benchmarks contain the injected semantic failures in HydraFragileBench.
- We do not claim the public hosted API exposes arbitrary OpenCypher writes.
- We do not fabricate a live HydraDB integration run without credentials.

The target is the **agent/orchestrator boundary surrounding HydraDB**, especially the execution-feedback loop documented in the cookbooks.
