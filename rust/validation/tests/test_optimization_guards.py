# SPDX-License-Identifier: EPL-2.0
"""Evidence-producing assertion checks must not silently run with -O."""
import os
from pathlib import Path
import subprocess
import sys
import unittest

SCRIPTS = Path(__file__).resolve().parents[1] / 'scripts'


class OptimizationGuards(unittest.TestCase):
    def reject_optimized(self, script, function, count):
        code = '''import importlib.util
spec=importlib.util.spec_from_file_location('guarded',%r)
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
try:
    getattr(module,%r)(*([None]*%d))
except RuntimeError:
    pass
else:
    raise SystemExit('optimized evidence check was allowed')
''' % (str(SCRIPTS / script), function, count)
        subprocess.run([sys.executable, '-O', '-c', code], check=True,
                       env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'))

    def test_inventory_assertions_not_disabled(self):
        self.reject_optimized('check_metamodel_inventory.py', 'check', 2)

    def test_compiled_registry_assertions_not_disabled(self):
        self.reject_optimized('check_compiled_metamodel.py', 'check', 2)

    def test_fixture_runner_assertions_not_disabled(self):
        self.reject_optimized('run_ledger_fixtures.py', 'run', 5)


if __name__ == '__main__':
    unittest.main()
