from test.helper import (
    command_factory,
    execute_add,
    get_status,
    wait_for_process,
)


def test_kill(daemon_setup):
    execute_add({'command': 'sleep 60'})
    command_factory('kill', {'remove': False})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_kill_remove(daemon_setup):
    execute_add({'command': 'sleep 60'})
    command_factory('kill', {'remove': True})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_kill_remove_resume(daemon_setup):
    # Add status
    execute_add({'command': 'sleep 2'})
    command_factory('kill', {'remove': True})
    status = get_status()
    assert status['status'] == 'paused'
    # Old process should be
    execute_add({'command': 'sleep 2'})
    command_factory('start')
    status = wait_for_process(1)
    assert status['status'] == 'running'
    assert status['data'][1]['status'] == 'done'


def test_stop(daemon_setup):
    execute_add({'command': 'sleep 60'})
    command_factory('stop', {'remove': False})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_stop_remove(daemon_setup):
    execute_add({'command': 'sleep 2'})
    command_factory('stop', {'remove': True})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_stop_remove_resume(daemon_setup):
    # Add status
    execute_add({'command': 'sleep 2'})
    command_factory('stop', {'remove': True})
    status = get_status()
    assert status['status'] == 'paused'
    # Old process should be
    execute_add({'command': 'sleep 2'})
    command_factory('start')
    status = wait_for_process(1)
    assert status['status'] == 'running'
    assert status['data'][1]['status'] == 'done'
