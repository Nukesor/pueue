from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


def test_restart_fails(daemon_setup):
    """Fail if restarting a non existent key."""
    response = command_factory('restart')({'keys': [0]})
    assert response['status'] == 'error'


def test_restart(daemon_setup):
    """Restart a command."""
    # Add command and let it finish
    execute_add('sleep 1')
    wait_for_process(0)

    # Restart the command. This should clone the entry and add it to the queue.
    response = command_factory('restart')({'keys': [0]})
    assert response['status'] == 'success'

    status = command_factory('status')()
    assert len(status['data']) == 2
    assert status['data'][1]['path'] == status['data'][0]['path']
    assert status['data'][1]['command'] == status['data'][0]['command']
    assert status['data'][1]['status'] == 'running'


def test_restart_running(daemon_setup):
    """Restart a running command fails."""
    execute_add('sleep 5')
    response = command_factory('restart')({'keys': [0]})
    assert response['status'] == 'error'

    # There is still only one running entry in the queue
    status = command_factory('status')()
    assert len(status['data']) == 1
    assert status['data'][0]['status'] == 'running'


def test_restart_multiple(daemon_setup):
    """Restart a running command fails."""
    execute_add('ls')
    execute_add('ls')
    execute_add('ls')
    execute_add('sleep 0.1')
    wait_for_process(3)

    # Restart the commands 0 and 3. This should clone the entries and add it to the queue.
    response = command_factory('pause')()
    response = command_factory('restart')({'keys': [0, 3]})
    assert response['status'] == 'success'

    status = command_factory('status')()
    assert len(status['data']) == 6
    assert status['data'][4]['path'] == status['data'][0]['path']
    assert status['data'][4]['command'] == status['data'][0]['command']
    assert status['data'][4]['status'] == 'queued'

    assert status['data'][5]['path'] == status['data'][3]['path']
    assert status['data'][5]['command'] == status['data'][3]['command']
    assert status['data'][5]['status'] == 'queued'
