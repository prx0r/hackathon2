#!/usr/bin/env python3
"""Evidence-first inspection of a checked-out HydraDB source tree.

Never guesses the native import command. It records exactly what the checked-out
revision exposes: Cargo features/binaries and source hits for Cypher/import terms.
"""
from __future__ import annotations
import argparse, hashlib, json, os, re, subprocess, tomllib
from pathlib import Path

TERMS = ["server-runtime", "nodes.jsonl", "edges.jsonl", "libcypher", "cypher", "GraphBLAS", "graphblas", "import"]

def git(root, *args):
    try:
        return subprocess.check_output(["git","-C",str(root),*args], text=True, stderr=subprocess.DEVNULL).strip()
    except Exception:
        return None

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("source")
    ap.add_argument("--out", default=None)
    ap.add_argument("--allow-missing-feature", action="store_true")
    args=ap.parse_args()
    root=Path(args.source).resolve()
    if not root.is_dir(): raise SystemExit(f"not a directory: {root}")
    cargo_files=list(root.rglob("Cargo.toml"))
    packages=[]; feature_hits=[]; bins=[]
    for p in cargo_files:
        try: d=tomllib.loads(p.read_text())
        except Exception: continue
        pkg=d.get("package",{})
        if pkg:
            packages.append({"path":str(p.relative_to(root)),"name":pkg.get("name"),"version":pkg.get("version")})
        feats=d.get("features",{})
        if "server-runtime" in feats:
            feature_hits.append({"path":str(p.relative_to(root)),"package":pkg.get("name"),"value":feats["server-runtime"]})
        for b in d.get("bin",[]) if isinstance(d.get("bin",[]),list) else []:
            bins.append({"manifest":str(p.relative_to(root)),"name":b.get("name"),"path":b.get("path")})
    hits={t:[] for t in TERMS}
    exts={".rs",".toml",".md",".json",".jsonl",".sh",".yaml",".yml"}
    for p in root.rglob("*"):
        if not p.is_file() or p.suffix not in exts: continue
        try: txt=p.read_text(errors="ignore")
        except Exception: continue
        for t in TERMS:
            if t.lower() in txt.lower():
                lines=[]
                for i,line in enumerate(txt.splitlines(),1):
                    if t.lower() in line.lower():
                        lines.append({"line":i,"text":line.strip()[:500]})
                        if len(lines)>=8: break
                hits[t].append({"path":str(p.relative_to(root)),"matches":lines})
                if len(hits[t])>=30: hits[t]=hits[t][:30]
    report={
        "schema":"iolaus.hydradb.source-discovery.v1",
        "root":str(root),"git_head":git(root,"rev-parse","HEAD"),"git_branch":git(root,"rev-parse","--abbrev-ref","HEAD"),
        "cargo_manifests":len(cargo_files),"packages":packages,"server_runtime_feature":feature_hits,"bins":bins,"source_hits":hits,
    }
    encoded=json.dumps(report,sort_keys=True,separators=(",",":")).encode(); report["report_sha256"]=hashlib.sha256(encoded).hexdigest()
    out=Path(args.out) if args.out else root/"iolaus-hydradb-source-discovery.json"
    out.write_text(json.dumps(report,indent=2,sort_keys=True)+"\n")
    print(json.dumps({"out":str(out),"git_head":report["git_head"],"server_runtime_feature_hits":len(feature_hits),"bins":bins},indent=2))
    if not feature_hits and not args.allow_missing_feature:
        raise SystemExit("FAIL: server-runtime feature not found in checked-out source; do not run a guessed build")

if __name__=="__main__": main()
