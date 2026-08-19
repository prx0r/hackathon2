#!/usr/bin/env python3
"""Static/result anti-cheat audit for Iolaus benchmark artifacts."""
from __future__ import annotations
import argparse, hashlib, json, re, sys
from pathlib import Path

FORBIDDEN_TASK_HINTS=('silent_noop','false_green','accepted_not_indexed','wrong_record','fault_injected')

def fail(msg): print('FAIL:',msg,file=sys.stderr); raise SystemExit(1)
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--root',default='.'); ap.add_argument('--result'); args=ap.parse_args()
    root=Path(args.root)
    tasks=json.loads((root/'fixtures/tasks/hydradb-live-tasks.json').read_text())['tasks']
    for s in tasks:
        for t in s['tasks']:
            low=t.lower()
            if any(x in low for x in FORBIDDEN_TASK_HINTS): fail(f"task leaks fault mechanism: {s['id']}")
    hydra=(root/'crates/iolaus-hydra/src/lib.rs').read_text()
    for required in ['iolaus-baseline-execution-log','iolaus-verified-execution-log']:
        # source constructs these dynamically; assert both arm labels exist instead.
        pass
    if 'let prefix = if outcome.arm == "baseline" { "baseline" } else { "verified" };' not in hydra:
        fail('feedback arm namespace separation missing')
    live=(root/'crates/iolaus-bench/src/live.rs').read_text()
    if 'same intended action and identical seeded fault' not in live: fail('paired-design declaration missing')
    if 'verified_false_positive_learning_signal' not in live: fail('learning contamination metric missing')
    if 'pub raw_response: Value' not in live: fail('raw Hydra response evidence is not retained')
    hermes=(root/'crates/iolaus-bench/src/hermes.rs').read_text()
    if 'pub raw_stdout: String' not in hermes: fail('raw Hermes stdout evidence is not retained')
    if args.result:
        r=json.loads(Path(args.result).read_text())
        if r.get('schema')=='iolaus.hydradb.livebench.v1':
            for t in r['trials']:
                p=t['paired_execution'];
                if p['baseline']['fault_injected'] != p['verified']['fault_injected']: fail('unpaired fault assignment')
                if t['feedback']['verified_false_positive_learning_signal']: fail('verified false-positive learning signal')
                if r.get('hydra_feedback_enabled') and not (t['feedback'].get('baseline_logged') and t['feedback'].get('verified_logged')):
                    fail('feedback-enabled live trial missing arm log evidence')
                raw_h=t['route'].get('raw_response')
                if not isinstance(raw_h, (dict,list)): fail('missing raw Hydra response evidence')
                raw_h_bytes=json.dumps(raw_h,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
                if hashlib.sha256(raw_h_bytes).hexdigest()!=t['route'].get('response_sha256'):
                    fail('raw Hydra response hash mismatch')
                raw_o=t['extraction'].get('raw_stdout')
                if not isinstance(raw_o, str): fail('missing raw Hermes stdout evidence')
                if hashlib.sha256(raw_o.encode()).hexdigest()!=t['extraction'].get('raw_stdout_sha256'):
                    fail('raw Hermes stdout hash mismatch')
                if p['verified'].get('receipt') and not p['verified']['postcondition_true']: fail('receipt attached to false state')
        elif r.get('schema')=='iolaus.hydrafragilebench.v1':
            for p in r['trials']:
                if p['baseline']['fault_injected'] != p['verified']['fault_injected']: fail('unpaired fault assignment')
        else: fail('unknown result schema')
    print('ANTI_CHEAT_AUDIT_PASS')
if __name__=='__main__': main()
