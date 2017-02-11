import pytest
from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


@pytest.mark.skip(reason="Needs to be adjusted to new multi process style")
def test_multiple_spawned(daemon_setup):
    """Pause the daemon."""

    # Add sleep command
    status = command_factory('config')({
        "option": "maxProcesses",
        "value": 3,
    })
    execute_add('sleep 60')
    execute_add('sleep 60')
    execute_add('sleep 60')
    execute_add('sleep 60')
    # Pause it with `'wait': True`
    # The paused daemon should wait for the process to finish
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'running'
    assert status['data'][1]['status'] == 'running'
    assert status['data'][2]['status'] == 'running'
    assert status['data'][0]['status'] == 'queued'

