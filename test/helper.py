import os
from time import sleep

from pueue.client.factories import command_factory as original_command_factory


def command_factory(command):
    function = original_command_factory(command)
    def test_communicate(body={}):
        current = os.getcwd()
        path = os.path.join(current, 'temptest')
        return function(body, path)
    return test_communicate


def execute_add(command):
    payload = {'command': command,
               'path': '/tmp'}
    command_factory('add')(payload)


def wait_for_process(key):
    status = command_factory('status')()
    while (key not in status['data']) or (status['data'][key]['status'] != 'done'):
        sleep(1)
        status = command_factory('status')()
    return status
