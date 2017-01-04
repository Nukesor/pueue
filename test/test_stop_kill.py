from test.helper import (
    execute_add,
    wait_for_process,
)
from pueue.client.factories import command_factory


def test_kill(daemon_setup):
    """Kill a running process."""
    execute_add({'command': 'sleep 60'})
    command_factory('kill')({'remove': False})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_kill_remove(daemon_setup):
    """Kill a running process and remove it afterwards."""
    execute_add({'command': 'sleep 60'})
    command_factory('kill')({'remove': True})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_kill_remove_resume(daemon_setup):
    """Everything works properly after remove killing a subprocess."""
    # Add status
    execute_add({'command': 'sleep 2'})
    command_factory('kill')({'remove': True})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    # Old process should be
    execute_add({'command': 'sleep 2'})
    command_factory('start')()
    status = wait_for_process(1)
    assert status['status'] == 'running'
    assert status['data'][1]['status'] == 'done'


def test_stop(daemon_setup):
    """Stop a running process."""
    execute_add({'command': 'sleep 60'})
    command_factory('stop')({'remove': False})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_stop_remove(daemon_setup):
    """Stop a running process and remove it afterwards."""
    execute_add({'command': 'sleep 2'})
    command_factory('stop')({'remove': True})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_stop_remove_resume(daemon_setup):
    """Everything works properly after remove stopping a subprocess."""
    # Add status
    execute_add({'command': 'sleep 2'})
    command_factory('stop')({'remove': True})
    status = command_factory('status')()
    assert status['status'] == 'paused'
    # Old process should be
    execute_add({'command': 'sleep 2'})
    command_factory('start')()
    status = wait_for_process(1)
    assert status['status'] == 'running'
    assert status['data'][1]['status'] == 'done'
