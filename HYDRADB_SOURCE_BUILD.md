# Source-built HydraDB lane

## Why this is separate from the API lane

A cloud/API integration test proves that Iolaus can use HydraDB's documented memory/recall primitives. It does not prove the locally cloned graph engine compiled with the desired Cypher runtime.

The release therefore requires distinct evidence for:

1. source build;
2. source process startup;
3. native fixture import;
4. post-import graph query/count verification;
5. documented API smoke against the endpoint actually used by LiveBench.

## Disk-space fix

The original build failed because `/tmp` was a ~3.8 GB tmpfs. The build helper forces:

```text
TMPDIR=/root/iolaus-hydradb-build/tmp
CARGO_TARGET_DIR=/root/iolaus-hydradb-build/target
```

and checks minimum free disk before compiling.

## Native dependencies

Expected from the external brief:

```text
libcypher-parser-dev
libgraphblas-dev
cmake
pkg-config
```

The build script also checks for a C/C++ build toolchain.

## Feature proof

`scripts/discover_hydradb_source.py` parses every Cargo manifest and must find a literal `server-runtime` feature in the checked-out revision before the default build proceeds.

This is preferable to trusting a copied command from an older branch.

## Import proof

The committed fixture uses a portable node/edge JSONL envelope. HydraDB source is authoritative for its actual loader schema.

The coding agent must:

1. search the source for loader/import examples;
2. use the exact source-compatible format or write a deterministic converter;
3. run the import;
4. query the graph independently;
5. output `{"nodes":43,"edges":19}` to the generic import adapter;
6. preserve the import/query logs.

No `PASS.json` is written otherwise.
