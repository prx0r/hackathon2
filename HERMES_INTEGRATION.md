# Hermes integration

HydraDB's public Chief-of-Staff cookbook explicitly keeps parameter extraction in the application layer and notes that the example LLM provider can be replaced. Iolaus uses local Hermes so the benchmark does not require a third-party LLM key.

## Adapter

`crates/iolaus-bench/src/hermes.rs`

Invocation shape:

```bash
hermes -z '<prompt>' --source tool --ignore-rules
```

Optional per-run pins:

```bash
export HERMES_PROVIDER=<provider configured on the host>
export HERMES_INFERENCE_MODEL=<model configured on the host>
export HERMES_BIN=/path/to/hermes
export IOLAUS_HERMES_TIMEOUT_MS=120000
```

The prompt contains only:

- expected function ID;
- function JSON parameter schema;
- user task.

It explicitly forbids function reselection, benchmark discussion, and fault inference.

## Recorded evidence

Each extraction stores:

- prompt SHA-256;
- schema SHA-256;
- raw stdout SHA-256;
- parsed JSON parameters;
- schema-valid boolean;
- validation errors;
- latency;
- configured model/provider labels when provided.

The raw stdout itself is not persisted in the result to keep artifacts smaller and avoid accidental sensitive context retention. The hash binds the result to the observed output. For a forensic run, pipe Hermes stdout into a separate append-only evidence log before parsing.

## Validation scope

The embedded validator checks the benchmark schemas' required keys and primitive JSON types. It is deliberately narrow and deterministic. If benchmark schemas become more complex, replace it with a full JSON Schema library and add adversarial tests before expanding claims.

## No LLM in the verifier

Hermes is never used for:

- deciding whether a CRM row exists;
- checking deployment health;
- confirming queue presence;
- checking processing status;
- checking evidence cardinality;
- matching fiscal period metadata;
- signing receipts.

Those remain deterministic. This is important: the benchmark is testing whether adding an independent verification boundary helps, not whether a second LLM agrees with the first.
