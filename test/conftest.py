import os
import pytest
import subprocess

from test.helper import command_factory
from pueue.helper.files import create_config_dir


@pytest.fixture(scope='function')
def daemon_setup(request):
    queue = create_config_dir()+'/queue'
    if os.path.exists(queue):
        os.remove(queue)

    process = subprocess.Popen(
        'pueue --daemon',
        shell=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    output, error = process.communicate()
    command_factory('reset')

    def daemon_teardown():
        command_factory('reset')
        command_factory('STOPDAEMON')
    request.addfinalizer(daemon_teardown)
