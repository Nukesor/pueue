from test.helper import *
from test.fixtures import *


def test_pause(daemon_setup):
    status = getStatus()
    assert status['status'] == 'running'
    commandFactory('pause')
    status = getStatus()
    assert status['status'] == 'paused'


def test_start(daemon_setup):
    commandFactory('pause')
    commandFactory('start')
    status = getStatus()
    assert status['status'] == 'running'


def test_kill(daemon_setup):
    executeAdd({'command': 'sleep 60'})
    commandFactory('kill', {'remove': False})
    status = getStatus()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_kill_remove(daemon_setup):
    executeAdd({'command': 'sleep 60'})
    commandFactory('kill', {'remove': True})
    status = getStatus()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_stop(daemon_setup):
    executeAdd({'command': 'sleep 60'})
    commandFactory('stop', {'remove': False})
    status = getStatus()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'


def test_stop_remove(daemon_setup):
    executeAdd({'command': 'sleep 60'})
    commandFactory('stop', {'remove': True})
    status = getStatus()
    assert status['status'] == 'paused'
    assert status['process'] == 'No running process'
    assert status['data'] == 'Queue is empty'


def test_status(daemon_setup):
    executeAdd({'command': 'sleep 60'})
    status = getStatus()
    assert status['status'] == 'running'
    assert status['process'] == 'running'


def test_reset_paused(daemon_setup):
    commandFactory('pause')
    executeAdd({'command': 'sleep 60'})
    executeAdd({'command': 'sleep 60'})
    commandFactory('reset')
    status = getStatus()
    assert status['status'] == 'paused'
    assert status['data'] == 'Queue is empty'


def test_reset_running(daemon_setup):
    commandFactory('start')
    executeAdd({'command': 'sleep 60'})
    executeAdd({'command': 'sleep 60'})
    commandFactory('reset')
    status = getStatus()
    assert status['status'] == 'running'
    assert status['data'] == 'Queue is empty'
