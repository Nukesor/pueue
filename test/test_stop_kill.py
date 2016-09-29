from test.helper import (
    command_factory,
    execute_add,
    get_status,
    wait_for_process,
)


def test_kill(daemon_setup):
    """Kill a running process."""
    execute_add({'command': 'sleep 60'})
    command_factory('kill', {'remove': False})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_kill_remove(daemon_setup):
    """Kill a running process and remove it afterwards."""
    execute_add({'command': 'sleep 60'})
    command_factory('kill', {'remove': True})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_kill_remove_resume(daemon_setup):
    """Everything works properly after remove killing a subprocess."""
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
    """Stop a running process."""
    execute_add({'command': 'sleep 60'})
    command_factory('stop', {'remove': False})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_stop_remove(daemon_setup):
    """Stop a running process and remove it afterwards."""
    execute_add({'command': 'sleep 2'})
    command_factory('stop', {'remove': True})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_stop_remove_resume(daemon_setup):
    """Everything works properly after remove stopping a subprocess."""
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
