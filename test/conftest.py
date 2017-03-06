import os
import time
import pytest
import subprocess

from test.helper import (
    execute_add,
    command_factory,
)


@pytest.fixture(scope='session')
def directory_setup(request):
    """Create the test directory and return root and config path."""
    current = os.getcwd()
    test_dir = os.path.join(current, 'temptest')
    if not os.path.exists(test_dir):
        os.mkdir(test_dir)

    config_dir = os.path.join(test_dir, '.config/pueue')

    return (test_dir, config_dir)


@pytest.fixture(scope='function')
def daemon_setup(request, directory_setup):
    """Start a daemon with a local test directory."""

    if os.path.exists(os.path.join(directory_setup[1], 'pueue.ini')):
        os.remove(os.path.join(directory_setup[1], 'pueue.ini'))

    process = subprocess.Popen(
        'pueue --daemon --root {}'.format(directory_setup[0]),
        shell=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    output, error = process.communicate()
    socket_path = os.path.join(directory_setup[1], 'pueue.sock')
    while not os.path.exists(socket_path):
        time.sleep(0.25)

    command_factory('reset')()

    def daemon_teardown():
        command_factory('reset')()
        command_factory('STOPDAEMON')()
    request.addfinalizer(daemon_teardown)


@pytest.fixture(scope='function')
def multiple_setup(daemon_setup):
    """Return a function which sets up a multiple process test case."""

    def setup(processes=4, max_processes=3, sleep_time=60):
        # Set max processes to three concurrent processes
        command_factory('config')({
            "option": "maxProcesses",
            "value": max_processes,
        })
        # Add sleep commands
        for i in range(processes):
            execute_add('sleep {}'.format(sleep_time))

    return setup
