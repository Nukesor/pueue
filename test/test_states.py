from test.helper import *
from test.fixtures import *


def test_pause(daemon_setup):
    status = get_status()
    assert status['status'] == 'running'
    command_factory('pause')
    status = get_status()
    assert status['status'] == 'paused'


def test_start(daemon_setup):
    command_factory('pause')
    command_factory('start')
    status = get_status()
    assert status['status'] == 'running'


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


def test_stop(daemon_setup):
    execute_add({'command': 'sleep 60'})
    command_factory('stop', {'remove': False})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_stop_remove(daemon_setup):
    execute_add({'command': 'sleep 60'})
    command_factory('stop', {'remove': True})
    status = get_status()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


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
