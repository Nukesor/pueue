from test.helper import (
    command_factory,
    execute_add,
    get_status,
    wait_for_process,
)


def test_pause(daemon_setup):
    status = get_status()
    assert status['status'] == 'running'
    command_factory('pause')
    status = get_status()
    assert status['status'] == 'paused'


def test_waiting_pause(daemon_setup):
    execute_add({'command': 'sleep 2'})
    status = get_status()
    assert status['status'] == 'running'
    command_factory('pause', {'wait': True})
    status = wait_for_process(0)
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'done'
