from test.helper import (
    command_factory,
    execute_add,
    get_status,
)


def test_start(daemon_setup):
    command_factory('pause')
    command_factory('start')
    status = get_status()
    assert status['status'] == 'running'


def test_status(daemon_setup):
    execute_add({'command': 'sleep 60'})
    status = get_status()
    assert status['status'] == 'running'
    assert status['process'] == 'running'


def test_reset_paused(daemon_setup):
    command_factory('pause')
    execute_add({'command': 'sleep 60'})
    execute_add({'command': 'sleep 60'})
    command_factory('reset')
    status = get_status()
    assert status['status'] == 'paused'
    assert status['data'] == 'Queue is empty'


def test_reset_running(daemon_setup):
    command_factory('start')
    execute_add({'command': 'sleep 60'})
    execute_add({'command': 'sleep 60'})
    command_factory('reset')
    status = get_status()
    assert status['status'] == 'running'
    assert status['data'] == 'Queue is empty'
