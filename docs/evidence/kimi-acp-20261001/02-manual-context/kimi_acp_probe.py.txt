#!/usr/bin/env python3
"""Explicitly authorized local Kimi integration capture; never executes proposed code."""

import argparse
import json
import os
from pathlib import Path
import queue
import signal
import subprocess
import sys
import tempfile
import threading
import time

from kimi_subagent import digest, write_json

ROOT = Path(__file__).resolve().parents[1]
BOOT = "You are a coding subagent of Codex. Remember worker marker ACP_CODER_82B7. Do not use any tool yet. Reply exactly ACP_BOOT_READY."
TASK = """Work only from this supplied synthetic code; do not read or edit files or run commands/tests:
def split_labels(text: str) -> list[str]:
    return text.split(',')
Propose a replacement that strips whitespace and drops blank labels. Before writing code, use AskUserQuestion to ask the coordinator how duplicate labels should be handled. Offer exactly two choices labeled 'Preserve duplicates' and 'Remove duplicates'. Wait for its answer, then return the proposed function as text and explicitly state that it was not executed. Preserve order in either case."""
FOLLOWUP = "Keep the chosen duplicate policy and add a one-line docstring to the proposed function. Return the worker marker remembered from the first message and the updated function. Do not use tools or execute the code."
CANCEL = "Use AskUserQuestion now to ask the coordinator to choose 'Option A' or 'Option B'. Wait for its response before doing anything else. This is an interruption exercise; do not read files or execute commands."
RECOVER = "The preceding turn was cancelled by the coordinator. Do not use tools. Reply exactly ACP_RECOVERED_82B7."
RELOAD = "Do not use tools. State the worker marker from the start of our conversation and the duplicate policy chosen by the coordinator."


class Client:
    def __init__(self, cwd, out, sid, timeout=180):
        self.out = out
        self.events = []
        self.queue = queue.Queue()
        self.err = out.parent.joinpath(out.name + '-client.stderr.txt').open('xb')
        self.proc = subprocess.Popen([
            sys.executable, '-B', str(ROOT / 'tools/kimi_acp_session.py'),
            '--session', sid, '--work-dir', str(cwd), '--output-dir', str(out),
            '--model', 'kimi-code/k3-256k', '--timeout', str(timeout),
        ], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.err, text=True)
        def reader():
            for line in self.proc.stdout:
                self.queue.put(json.loads(line))
            self.queue.put(None)
        self.reader = threading.Thread(target=reader, daemon=True)
        self.reader.start()

    def wait(self, event, seconds=55):
        until = time.monotonic() + seconds
        while True:
            value = self.queue.get(timeout=max(.01, until - time.monotonic()))
            if value is None:
                raise RuntimeError(f'client ended before {event}; inspect {self.out}')
            self.events.append(value)
            if value['event'] == event:
                return value
            if event == 'question' and value['event'] == 'turn_end':
                raise RuntimeError('turn completed without the required reverse-RPC question')
            if value['event'] == 'closed':
                raise RuntimeError(f'closed before {event}: {value}')

    def send(self, command, **fields):
        self.proc.stdin.write(json.dumps({'command': command, **fields}) + '\n')
        self.proc.stdin.flush()

    def cleanup(self):
        if self.proc.poll() is None:
            self.proc.send_signal(signal.SIGTERM)
        self.proc.wait(timeout=15)
        self.reader.join(timeout=2)
        self.err.close()
        write_json(self.out.parent / (self.out.name + '-controller.json'),
                   {'exit_code': self.proc.returncode, 'observed_events': self.events})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', required=True, type=Path)
    args = parser.parse_args()
    out = args.output_dir.absolute()
    out.mkdir(parents=True, exist_ok=False)
    write_json(out / 'plan.json', {'scope': 'synthetic proposal and ACP lifecycle only',
        'owned_files': [], 'model': 'kimi-code/k3-256k',
        'driver_sha256': digest(Path(__file__).read_bytes()),
        'prompts': {'bootstrap': BOOT, 'task': TASK, 'followup': FOLLOWUP,
                    'cancel': CANCEL, 'recover': RECOVER, 'reload': RELOAD}})
    scratch = None
    checks = {}
    error = None
    try:
        with tempfile.TemporaryDirectory(prefix='codex-kimi-acp-') as path:
            scratch = Path(path)
            write_json(out / 'scratch.json', {'path': path, 'owned_by': 'Kimi ACP integration probe',
                'contents_before': [], 'retirement': 'context manager exit, before handoff'})
            prompt = out / 'bootstrap-prompt.txt'
            prompt.write_text(BOOT)
            boot = subprocess.run([sys.executable, '-B', str(ROOT / 'tools/kimi_subagent.py'),
                '--model', 'kimi-code/k3-256k', '--agent-file', str(ROOT / '.agents/kimi/interactive.md'),
                '--prompt-file', str(prompt), '--work-dir', path,
                '--output-dir', str(out / '01-bootstrap'), '--timeout', '60'],
                capture_output=True, text=True, timeout=85)
            (out / 'bootstrap-controller.stdout.txt').write_text(boot.stdout)
            (out / 'bootstrap-controller.stderr.txt').write_text(boot.stderr)
            if boot.returncode:
                raise RuntimeError(f'bootstrap returned {boot.returncode}')
            rows = [json.loads(line) for line in (out / '01-bootstrap/stdout.jsonl').read_text().splitlines()]
            sid = next(row['session_id'] for row in rows if row.get('type') == 'session.resume_hint')
            print(json.dumps({'event': 'bootstrap', 'session_id': sid}), flush=True)
            c = Client(scratch, out / '02-dialogue', sid)
            try:
                c.wait('ready')
                c.send('prompt', text=TASK)
                question = c.wait('question')
                schema = question['requestedSchema']
                name = schema['required'][0]
                assert 'Preserve duplicates' in [item['const'] for item in schema['properties'][name]['oneOf']]
                c.send('answer', request_id=question['request_id'], content={name: 'Preserve duplicates'})
                checks['question_turn'] = c.wait('turn_end')
                c.send('prompt', text=FOLLOWUP)
                checks['followup_turn'] = c.wait('turn_end')
                c.send('prompt', text=CANCEL)
                c.wait('question')
                c.send('prompt', text='This must be rejected while the current turn is waiting.')
                checks['busy_rejected'] = c.wait('command_error')
                c.send('cancel')
                checks['cancelled_turn'] = c.wait('turn_end')
                assert checks['cancelled_turn'].get('stopReason') == 'cancelled'
                c.send('prompt', text=RECOVER)
                checks['recovered_turn'] = c.wait('turn_end')
                c.send('close')
                checks['closed'] = c.wait('closed')
                assert checks['closed']['status'] == 'closed'
            finally:
                c.cleanup()
            c = Client(scratch, out / '03-reload', sid)
            try:
                c.wait('ready')
                c.send('prompt', text=RELOAD)
                checks['reload_turn'] = c.wait('turn_end')
                c.proc.stdin.close()
                checks['eof_close'] = c.wait('closed')
            finally:
                c.cleanup()
            c = Client(scratch, out / '04-deadline', sid, timeout=3)
            try:
                checks['deadline'] = c.wait('closed')
                assert checks['deadline']['status'] == 'timeout'
            finally:
                c.cleanup()
            c = Client(scratch, out / '05-signal', sid)
            try:
                c.wait('ready')
                os.kill(c.proc.pid, signal.SIGTERM)
                checks['signal'] = c.wait('closed')
                assert checks['signal']['status'] == 'interrupted'
            finally:
                c.cleanup()
            checks['scratch_contents_after'] = sorted(str(p.relative_to(scratch)) for p in scratch.rglob('*'))
    except Exception as exc:
        error = f'{type(exc).__name__}: {exc}'
    finally:
        write_json(out / 'capture.json', {'checks': checks, 'error': error,
            'scratch_path': str(scratch), 'scratch_removed': scratch is not None and not scratch.exists()})
    print(json.dumps({'event': 'capture_finished', 'error': error, 'checks': list(checks)}), flush=True)
    return 1 if error else 0


if __name__ == '__main__':
    raise SystemExit(main())
