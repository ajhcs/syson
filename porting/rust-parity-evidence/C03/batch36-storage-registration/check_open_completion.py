"""Check the documented OPEN/exit-2 result without claiming feature acceptance."""
import json
import subprocess
import sys

arguments = [sys.executable, '-s', '-B']
if sys.flags.optimize:
    arguments.append('-O')
arguments += ['tools/verify_plan.py', '--check-completion']
result = subprocess.run(arguments, capture_output=True, text=True, timeout=120)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
report = json.loads(result.stdout)
if (result.returncode != 2 or report.get('parity') != 'OPEN'
        or type(report.get('open_gate_count')) is not int or report['open_gate_count'] <= 0):
    raise ValueError('expected actual completion checker OPEN with exit 2')
print(json.dumps({'expected_completion_exit': result.returncode, 'feature_accepted': False}))
