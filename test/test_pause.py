import time
from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


def test_pause(daemon_setup):
    """Pause the daemon."""
    status = command_factory('status')()
    assert status['status'] == 'running'
    command_factory('pause')()
    status = command_factory('status')()
    assert status['status'] == 'paused'


def test_waiting_pause(daemon_setup):
    """Daemon waits for process to finish.

    With `wait=True` as a parameter the daemon pauses,
    but waits for the current process to finish instead of
    pausing it.
    """
    # Add sleep command
    execute_add('sleep 2')
    status = command_factory('status')()
    assert status['status'] == 'running'
    # Pause it with `'wait': True`
    command_factory('pause')({'wait': True})
    # The paused daemon should wait for the process to finish
    status = wait_for_process(0)
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'done'


def test_start_after_pause(daemon_setup):
    """Daemon really pauses subprocess.

    In case the subprocess doesn't pause, the command should complete instantly
    after starting the daemon again.
    """
    # Add command and make sure it's running
    execute_add('sleep 2 && sleep 2')
    status = command_factory('status')()
    assert status['status'] == 'running'

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
    assert status['status'] == 'running'
    assert status['data'][0]['status'] == 'done'
