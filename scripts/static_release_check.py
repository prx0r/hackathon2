#!/usr/bin/env python3
from pathlib import Path
import json, tomllib, sys
root=Path(sys.argv[1] if len(sys.argv)>1 else '.')
for p in root.rglob('*.toml'):
    with p.open('rb') as f: tomllib.load(f)
for p in root.rglob('*.json'):
    # result directories may be empty; all committed JSON must parse
    json.loads(p.read_text())
for p in root.rglob('*.rs'):
    s=p.read_text()
    if 'TODO' in s or 'FIXME' in s: raise SystemExit(f'placeholder marker: {p}')
    if s.count('{')!=s.count('}'): raise SystemExit(f'rough brace mismatch: {p}')
print('STATIC_RELEASE_CHECK_PASS')
