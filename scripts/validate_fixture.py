#!/usr/bin/env python3
from __future__ import annotations
import json, sys
from pathlib import Path
root=Path(sys.argv[1] if len(sys.argv)>1 else 'fixtures/hydradb')
nodes=[json.loads(x) for x in (root/'nodes.jsonl').read_text().splitlines() if x.strip()]
edges=[json.loads(x) for x in (root/'edges.jsonl').read_text().splitlines() if x.strip()]
ids=[n['id'] for n in nodes]
assert len(ids)==len(set(ids)), 'duplicate node ids'
assert all(e['from'] in set(ids) and e['to'] in set(ids) for e in edges), 'orphan edge'
assert all('fault' not in json.dumps(n).lower() for n in nodes), 'fixture leaks fault labels'
manifest=json.loads((root/'manifest.json').read_text())
assert len(nodes)==manifest['nodes'] and len(edges)==manifest['edges']
print(json.dumps({'status':'PASS','nodes':len(nodes),'edges':len(edges)},indent=2))
