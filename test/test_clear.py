from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


def test_kill_remove_resume(daemon_setup):
    """Old `done` and `failed` entries will be deleted."""

    # Add a command that fails, and finishes as well as queued and running processes
    execute_add('failingstufftest')
    execute_add('ls')
    execute_add('ls')
    execute_add('sleep 60')
    execute_add('ls')
    status = wait_for_process(2)

    # Trigger the clear
    command_factory('clear')()

    # Check that 0,1,2 are missing, 3 is 'running' and 4 is 'queued'
    status = command_factory('status')()
    assert 0 not in status['data']
    assert 1 not in status['data']
    assert 2 not in status['data']
    assert status['data'][3]['status'] == 'running'
    assert status['data'][4]['status'] == 'queued'
