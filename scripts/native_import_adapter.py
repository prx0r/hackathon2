#!/usr/bin/env python3
"""Run the exact native HydraDB import and count-verification commands for a revision.

The adapter is intentionally command-agnostic. A coding agent must inspect the
checked-out HydraDB source and supply argv arrays; this script refuses to guess.

Example environment shape (illustrative only, not HydraDB syntax):
  HYDRADB_IMPORT_ARGV_JSON='["/path/server","import","--nodes","{nodes}","--edges","{edges}"]'
  HYDRADB_VERIFY_ARGV_JSON='["/path/query-tool","...args..."]'
The verify command MUST emit a JSON object with integer keys `nodes` and `edges`.
"""
from __future__ import annotations
import argparse, hashlib, json, os, subprocess, time
from pathlib import Path

def parse_argv(name:str, mapping:dict[str,str]):
    raw=os.environ.get(name)
    if not raw:
        raise SystemExit(f"FAIL-CLOSED: {name} is unset; inspect this HydraDB revision and provide exact argv JSON")
    try: xs=json.loads(raw)
    except Exception as e: raise SystemExit(f"{name} must be a JSON array: {e}")
    if not isinstance(xs,list) or not all(isinstance(x,str) for x in xs):
        raise SystemExit(f"{name} must be a JSON string array")
    out=[]
    for x in xs:
        for key,value in mapping.items():
            x=x.replace("{"+key+"}",value)
        out.append(x)
    return out

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--fixture',default='fixtures/hydradb')
    ap.add_argument('--evidence-dir',default='evidence/native-import')
    args=ap.parse_args()
    fixture=Path(args.fixture).resolve(); ev=Path(args.evidence_dir).resolve(); ev.mkdir(parents=True,exist_ok=True)
    nodes=fixture/'nodes.jsonl'; edges=fixture/'edges.jsonl'; manifest=json.loads((fixture/'manifest.json').read_text())
    mapping={'nodes':str(nodes),'edges':str(edges),'fixture':str(fixture)}
    import_argv=parse_argv('HYDRADB_IMPORT_ARGV_JSON',mapping)
    verify_argv=parse_argv('HYDRADB_VERIFY_ARGV_JSON',mapping)
    (ev/'import-argv.json').write_text(json.dumps(import_argv,indent=2)+'\n')
    (ev/'verify-argv.json').write_text(json.dumps(verify_argv,indent=2)+'\n')
    t0=time.time(); imp=subprocess.run(import_argv,text=True,capture_output=True)
    (ev/'import.stdout.log').write_text(imp.stdout); (ev/'import.stderr.log').write_text(imp.stderr)
    if imp.returncode:
        raise SystemExit(f"native import failed rc={imp.returncode}; see {ev}")
    vr=subprocess.run(verify_argv,text=True,capture_output=True)
    (ev/'verify.stdout.log').write_text(vr.stdout); (ev/'verify.stderr.log').write_text(vr.stderr)
    if vr.returncode:
        raise SystemExit(f"native verification query failed rc={vr.returncode}; see {ev}")
    try: counts=json.loads(vr.stdout.strip())
    except Exception as e: raise SystemExit(f"verify command must emit JSON only: {e}")
    want={'nodes':manifest['nodes'],'edges':manifest['edges']}
    got={'nodes':int(counts.get('nodes',-1)),'edges':int(counts.get('edges',-1))}
    if got!=want: raise SystemExit(f"count mismatch: expected {want}, got {got}")
    record={'schema':'iolaus.native-import-proof.v1','expected':want,'observed':got,'elapsed_s':round(time.time()-t0,3),
            'nodes_sha256':hashlib.sha256(nodes.read_bytes()).hexdigest(),'edges_sha256':hashlib.sha256(edges.read_bytes()).hexdigest()}
    (ev/'PASS.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    print(json.dumps(record,indent=2)); print('NATIVE_HYDRADB_IMPORT_PASS')
if __name__=='__main__': main()
