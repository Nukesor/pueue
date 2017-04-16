import time
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
    assert status['data'][0]['status'] == 'failed'


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
    assert status['data'][0]['status'] == 'failed'
    assert status['data'][1]['status'] == 'failed'
    assert status['data'][2]['status'] == 'failed'
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


def test_kill_single(daemon_setup):
    """Kill a running process and check if it finishes as failed."""
    # We need to add some bash syntax with "&&" otherwise python won't spawn a
    # shell parent process. But we need this for debugging.
    execute_add('sleep 60 && ls')

    # Unfortunately this is necessary as the shell parent process needs some time to spawn it's children
    time.sleep(1)
    # Kill the children of the parent process
    status = command_factory('kill')({'keys': [0], 'signal': 'sigkill'})
    status = command_factory('status')()
    status = wait_for_process(0)
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'failed'


def test_kill_single_with_multiple_commands(daemon_setup):
    """Assure that the signal will be only send to the shells child processes."""
    # Once the first sleep finishes the second sleep process will be spawned.
    # By this we can assure that the signal is only sent to the child processes.
    execute_add('sleep 60 ; sleep 60')

    # Unfortunately this is necessary as the shell parent process needs some time to spawn it's children
    time.sleep(1)

    # Kill the children of the parent process
    status = command_factory('kill')({'keys': [0], 'signal': 'sigkill'})

    # Give the shell process some time to clean the old process and spawn the new one.
    time.sleep(1)

    # Assert that the queue entry is still running
    status = command_factory('status')()
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'running'


def test_kill_shell_of_single_with_multiple_commands(daemon_setup):
    """Assure that the signal will be sent to the shell process with '-a' provided."""
    # Once the first sleep finishes the second sleep process will be spawned.
    # By this we can assure that the signal is only sent to the child processes.
    execute_add('sleep 60 ; sleep 60')

    # Unfortunately this is necessary as the shell parent process needs some time to spawn it's children
    time.sleep(1)

    # Kill the shell process as well as the child processes.
    status = command_factory('kill')({'keys': [0], 'signal': 'sigkill', 'all': True})

    # Give the shell process some time to die and pueue to clean up the mess.
    time.sleep(1)

    # Assert that the queue entry is finished and failed
    status = command_factory('status')()
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'failed'
