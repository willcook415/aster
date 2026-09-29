"""Export matched Criterion baselines without rounding away the original estimates."""
import argparse
import csv
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--root', type=Path, default=Path('target/criterion'))
parser.add_argument('--before', default='before-cache')
parser.add_argument('--after', default='after-cache')
parser.add_argument('--output', type=Path, default=Path('docs/benchmarks/2026-09-29-cache.csv'))
args = parser.parse_args()
rows = []
for before in sorted(args.root.glob(f'**/{args.before}/estimates.json')):
    after = before.parent.parent / args.after / 'estimates.json'
    if not after.exists():
        continue
    name = json.loads((before.parent / 'benchmark.json').read_text())['full_id']
    a = json.loads(before.read_text())['mean']
    b = json.loads(after.read_text())['mean']
    rows.append(dict(workload=name, before_mean_ns=a['point_estimate'],
        before_lower_ns=a['confidence_interval']['lower_bound'],before_upper_ns=a['confidence_interval']['upper_bound'],
        after_mean_ns=b['point_estimate'],after_lower_ns=b['confidence_interval']['lower_bound'],
        after_upper_ns=b['confidence_interval']['upper_bound'],ratio=a['point_estimate']/b['point_estimate']))
if not rows:
    raise SystemExit('No matching baselines found')
args.output.parent.mkdir(parents=True,exist_ok=True)
with args.output.open('w',newline='',encoding='utf-8') as output:
    writer=csv.DictWriter(output,fieldnames=list(rows[0]))
    writer.writeheader()
    writer.writerows(rows)
for row in rows:
    print(f"{row['workload']}: {row['before_mean_ns']/1e6:.3f} -> {row['after_mean_ns']/1e6:.3f} ms ({row['ratio']:.2f}x)")
