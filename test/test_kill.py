import pytest

from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


@pytest.mark.parametrize('signal',
                         ['sigint', 'SIGINT', 'int', 'INT', '2',
                          'sigterm', 'SIGTERM', 'term', 'TERM', '15',
                          'sigkill', 'SIGKILL', 'kill', 'KILL', '9']
                         )
def test_kill(daemon_setup, signal):
    """Kill a running process."""
    execute_add('sleep 60')
    command_factory('kill')({'signal': signal})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'queued' or 'killing'


@pytest.mark.parametrize('signal', ['sigterm', 'sigint', 'sigkill'])
def test_kill_all(daemon_setup, multiple_setup, signal):
    """Kill all running processes."""
    # Setup multiple processes test case
    multiple_setup(
        max_processes=3,
        processes=4,
        sleep_time=60,
    )

    command_factory('kill')({'signal': signal})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'failed' or 'killing'
    assert status['data'][1]['status'] == 'failed' or 'killing'
    assert status['data'][2]['status'] == 'failed' or 'killing'
    assert status['data'][3]['status'] == 'queued'


@pytest.mark.parametrize('signal', ['sigterm', 'sigint', 'sigkill'])
def test_kill_multiple(daemon_setup, multiple_setup, signal):
    """Kill multiple running processes."""
    # Setup multiple processes test case
    multiple_setup(
        max_processes=3,
        processes=4,
        sleep_time=60,
    )

    # Only kill two of three running processes and wait for them being processed.
    command_factory('kill')({'keys': [0, 2], 'signal': signal})
    status = wait_for_process(2)

    # Two should be failed, and two should be running
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'failed'
    assert status['data'][1]['status'] == 'running'
    assert status['data'][2]['status'] == 'failed'
    assert status['data'][3]['status'] == 'running'


@pytest.mark.parametrize('signal', ['sigterm', 'sigint', 'sigkill'])
def test_kill_single(daemon_setup, signal):
    """Kill a running process and check if it finishes as failed."""
    execute_add('sleep 60')
    command_factory('kill')({'keys': [0], 'signal': signal})
    status = command_factory('status')()
    status = wait_for_process(0)
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'failed'
