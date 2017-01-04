from time import sleep

from pueue.client.factories import command_factory


def execute_add(command):
    payload = {'command': command,
               'path': '/tmp'}
    command_factory('add')(payload)


def wait_for_process(key):
    status = command_factory('status')()
    while (key not in status['data']) or (status['data'][key]['status'] != 'done'):
        sleep(1)
        status = command_factory('status')()
        print(status)
    return status
