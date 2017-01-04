import os
import pytest
import subprocess

from test.helper import command_factory


@pytest.fixture(scope='function')
def daemon_setup(request):
    current = os.getcwd()
    test_dir = os.path.join(current, 'temptest')

    process = subprocess.Popen(
        'pueue --daemon --root {}'.format(test_dir),
        shell=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    output, error = process.communicate()
    command_factory('reset')

    def daemon_teardown():
        command_factory('STOPDAEMON')
    request.addfinalizer(daemon_teardown)
