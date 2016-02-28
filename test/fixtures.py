import os
import pytest
import subprocess

from test.helper import commandFactory
from pueue.helper.files import createConfigDir


@pytest.fixture
def daemon_setup(request):
    queue = createConfigDir()+'/queue'
    if os.path.exists(queue):
        os.remove(queue)

    process = subprocess.Popen(
        'pueue --daemon',
        shell=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    output, error = process.communicate()
    commandFactory('reset')

    def daemon_teardown():
        commandFactory('reset')
        commandFactory('STOPDAEMON')
    request.addfinalizer(daemon_teardown)
