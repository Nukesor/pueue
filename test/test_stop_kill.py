from test.helper import (
    execute_add,
    wait_for_process,
    command_factory,
)


def test_kill(daemon_setup):
    """Kill a running process."""
    execute_add('sleep 60')
    command_factory('kill')()
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'queued'


def test_kill_remove(daemon_setup):
    """Kill a running process and remove it afterwards."""
    execute_add('sleep 60')
    command_factory('kill')({'remove': True, 'key': 0})
    status = command_factory('status')()
    assert status['status'] == 'running'
    assert status['data'] == 'Queue is empty'


def test_kill_remove_resume(daemon_setup):
    """Everything works properly after remove killing a subprocess."""
    # Add new command and kill it with remove flag set
    execute_add('sleep 60')
    command_factory('kill')({'remove': True, 'key': 0})

    # Old process is removed and new process should be running fine
    execute_add('sleep 1')
    status = wait_for_process(1)
    assert status['data'][1]['status'] == 'done'
    assert status['data'][1]['command'] == 'sleep 1'


def test_stop(daemon_setup):
    """Stop a running process."""
    execute_add('sleep 60')
    command_factory('stop')()
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'queued'


def test_stop_remove(daemon_setup):
    """Stop a running process and remove it afterwards."""
    execute_add('sleep 2')
    command_factory('stop')({'remove': True, 'key': 0})
    status = command_factory('status')()
    assert status['status'] == 'running'
    assert status['data'] == 'Queue is empty'


def test_stop_remove_resume(daemon_setup):
    """Everything works properly after remove stopping a subprocess."""
    # Add status
    execute_add('sleep 2')
    command_factory('stop')({'remove': True, 'key': 0})

    # Old process is removed and new process should be running fine
    execute_add('sleep 1')
    status = wait_for_process(1)
    assert status['data'][1]['status'] == 'done'
    assert status['data'][1]['command'] == 'sleep 1'
