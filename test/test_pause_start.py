import time
from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


def test_pause(daemon_setup):
    """Pause the daemon."""
    # Assert that the daemon is running after setup
    status = command_factory('status')()
    assert status['status'] == 'running'

    # Add a single command
    execute_add('sleep 60')
    command_factory('pause')()
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'paused'


def test_pause_multiple(daemon_setup, multiple_setup):
    """Pause the daemon with multiple running processes."""
    # Add a single command

    # Setup multiple processes test case
    multiple_setup(
        max_processes=3,
        processes=4,
        sleep_time=60,
    )
    # Pause and check if all running processes have been paused
    command_factory('pause')()
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'paused'
    assert status['data'][1]['status'] == 'paused'
    assert status['data'][2]['status'] == 'paused'
    assert status['data'][3]['status'] == 'queued'


def test_waiting_pause(daemon_setup):
    """Daemon waits for process to finish.

    With `wait=True` as a parameter the daemon pauses,
    but waits for the current process to finish instead of
    pausing it.
    """
    # Add sleep command and pause it with `'wait': True`
    execute_add('sleep 2')
    command_factory('pause')({'wait': True})
    # The paused daemon should wait for the process to finish
    status = wait_for_process(0)
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'done'


def test_waiting_pause_multiple(daemon_setup, multiple_setup):
    """Daemon waits for processes to finish.

    With `wait=True` as a parameter the daemon pauses,
    but waits for the current processes to finish instead of
    pausing them.
    """
    # Setup multiple processes test case
    multiple_setup(
        max_processes=2,
        processes=3,
        sleep_time=3,
    )

    # Pause the daemon with `'wait': True`
    command_factory('pause')({'wait': True})
    # The paused daemon should wait for the processes to finish
    status = wait_for_process(0)
    status = wait_for_process(1)
    assert status['data'][0]['status'] == 'done'
    assert status['data'][1]['status'] == 'done'
    assert status['data'][2]['status'] == 'queued'


def test_start_after_pause(daemon_setup):
    """Daemon really pauses subprocess.

    In case the subprocess doesn't pause, the command should complete instantly
    after starting the daemon again.
    """
    # Add command
    execute_add('sleep 4')

    # Pause the daemon and the process
    command_factory('pause')({'wait': False})
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'paused'
    time.sleep(5)

    # Start the daemon again assert that it is still running
    command_factory('start')()
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'running'

    # Wait for the process to finish
    status = wait_for_process(0)
    assert status['data'][0]['status'] == 'done'


def test_start_multiple_after_pause(daemon_setup, multiple_setup):
    """Daemon properly starts paused subprocesses."""
    # Setup multiple processes test case
    multiple_setup(
        max_processes=2,
        processes=2,
        sleep_time=3,
    )

    # Pause the daemon and the process
    command_factory('pause')({'wait': False})
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'paused'
    assert status['data'][1]['status'] == 'paused'

    # Start the daemon again assert that it is still running
    command_factory('start')()
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'running'
    assert status['data'][1]['status'] == 'running'

    # Wait for the process to finish
    status = wait_for_process(0)
    status = wait_for_process(1)
    assert status['data'][0]['status'] == 'done'
    assert status['data'][1]['status'] == 'done'


def test_multiple_start_during_pause(daemon_setup, multiple_setup):
    """It's possible to start processes, while the daemon is paused.

    This test checks if it's possible to start paused or queued tasks,
    even if the daemon is paused and doesn't process the queue.
    """
    # Setup multiple processes test case
    multiple_setup(
        max_processes=2,
        processes=4,
        sleep_time=60,
    )

    # Pause all running processes and the daemon
    command_factory('pause')()
    status = command_factory('status')()

    # Start the specific processes 0 and 3
    command_factory('start')({'key': 0})
    command_factory('start')({'key': 3})

    # Check if 0 is running, 1 should be still paused and 3 should be running
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'running'
    assert status['data'][1]['status'] == 'paused'
    assert status['data'][2]['status'] == 'queued'
    assert status['data'][3]['status'] == 'running'

    # Start 2 and check if it's running
    command_factory('start')({'key': 2})
    status = command_factory('status')()
    assert status['data'][2]['status'] == 'running'


def test_start_additional_process(daemon_setup):
    """It's possible to start a task, even if max_processes is exceeded."""
    execute_add('sleep 60')
    execute_add('sleep 60')
    execute_add('sleep 60')
    execute_add('sleep 60')

    status = command_factory('status')()
    # Start the specific processes 0 and 3
    command_factory('start')({'key': 3})

    # Check if 0 is running, 1 should be still paused and 3 should be running
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'running'
    assert status['data'][1]['status'] == 'queued'
    assert status['data'][2]['status'] == 'queued'
    assert status['data'][3]['status'] == 'running'

    # Start 2 and check if it's running
    command_factory('start')({'key': 2})
    status = command_factory('status')()
    assert status['data'][2]['status'] == 'running'
