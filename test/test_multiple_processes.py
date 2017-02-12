import pytest
from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


def test_multiple_spawned(daemon_setup):
    """Pause the daemon."""

    # Set max processes to three concurrent processes
    command_factory('config')({
        "option": "maxProcesses",
        "value": 3,
    })
    # Add sleep commands
    execute_add('sleep 60')
    execute_add('sleep 60')
    execute_add('sleep 60')
    execute_add('sleep 60')
    # Pause it with `'wait': True`
    # The paused daemon should wait for the process to finish
    status = command_factory('status')()
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'running'
    assert status['data'][1]['status'] == 'running'
    assert status['data'][2]['status'] == 'running'
    assert status['data'][3]['status'] == 'queued'

