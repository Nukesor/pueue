import pytest
from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


def test_multiple_spawned(daemon_setup, multiple_setup):
    """Check if multiple processes are running."""

    # Setup multiple processes test case
    multiple_setup(
        max_processes=3,
        processes=4,
        sleep_time=60,
    )
    # The fixture `multiple_setup` adds 4 new commands and sets
    # the amount of concurrent processes to 3.
    status = command_factory('status')()
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'running'
    assert status['data'][1]['status'] == 'running'
    assert status['data'][2]['status'] == 'running'
    assert status['data'][3]['status'] == 'queued'
