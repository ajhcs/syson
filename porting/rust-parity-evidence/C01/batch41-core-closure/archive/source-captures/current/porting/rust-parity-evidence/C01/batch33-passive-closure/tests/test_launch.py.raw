"""Actual pipe/drain checks; no network, systemd, or acquired code."""
import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import unittest

SPEC = importlib.util.spec_from_file_location('closure_launch', Path(__file__).parents[1] / 'launch.py')
launch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(launch)


@contextlib.contextmanager
def child(code):
    process = subprocess.Popen([sys.executable, '-I', '-B', '-c', code],
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        yield process
    finally:
        if process.poll() is None:
            process.kill()
        process.wait(timeout=5)
        process.stdout.close()
        process.stderr.close()


class CaptureTests(unittest.TestCase):
    def test_both_streams_are_retained_and_reaped(self):
        stdout, stderr = io.BytesIO(), io.BytesIO()
        with child("import sys; print('out'); print('err', file=sys.stderr)") as process:
            launch.capture(process, stdout, stderr)
            self.assertEqual(process.returncode, 0)
        self.assertEqual((stdout.getvalue(), stderr.getvalue()), (b'out\n', b'err\n'))

    def test_overflow_preserves_bounded_prefix_and_refuses(self):
        stdout, stderr = io.BytesIO(), io.BytesIO()
        with child("import sys; sys.stdout.write('x' * 2000000)") as process:
            with self.assertRaisesRegex(ValueError, 'controller log budget'):
                launch.capture(process, stdout, stderr)
        self.assertEqual(stdout.getvalue(), b'x' * launch.LOG_BYTES)
        self.assertEqual(stderr.getvalue(), b'')

    def test_silent_child_cannot_escape_controller_deadline(self):
        with child('import time; time.sleep(5)') as process:
            with self.assertRaises((ValueError, subprocess.TimeoutExpired)):
                launch.capture(process, io.BytesIO(), io.BytesIO(), seconds=0.02)
