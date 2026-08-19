# Primary implementation references

These links are inputs to the integration design, not evidence that an external release gate passed.

## HydraDB

- Documentation: https://docs.hydradb.com/
- API reference: https://docs.hydradb.com/api-reference/
- Chief of Staff cookbook: https://docs.hydradb.com/cookbooks/hydradb-cookbook-06
- Financial analyst cookbook: https://docs.hydradb.com/cookbooks/cookbook-10-ai-financial-analyst
- Competitive intelligence cookbook: https://docs.hydradb.com/cookbooks/competitive-intelligence-agent
- Public CLI: https://github.com/hydradatabase/hydradb-cli

Design facts used here:

1. Function schemas can be stored as HydraDB knowledge and retrieved with full recall.
2. Parameter extraction is an application-layer LLM step; the benchmark replaces cloud extraction with local Hermes.
3. Execution outcomes can be written back as memory/self-improvement signal, motivating a verified-success boundary.
4. Ingestion and processing/readiness are distinct operations, motivating accepted-but-not-indexed tests.

The source-built `graph-node` / `server-runtime` checkout described by the external brief was not publicly discoverable from this packaging environment. Its exact launch/import syntax MUST therefore be discovered from the checked-out revision and recorded by the release agent rather than guessed here.

## Hermes

- Function calling repo: https://github.com/NousResearch/Hermes-Function-Calling
- Hermes agent repo/CLI: https://github.com/NousResearch/hermes-agent

The live adapter calls local one-shot Hermes and constrains it to one JSON object for a supplied function schema. The verifier itself remains deterministic.
