import os
import pickle
import shutil
import unittest
import subprocess

from daemonize import Daemonize

from pueue.daemon.daemon import Daemon
from pueue.helper.socket import getClientSocket

from pueue.subcommands.daemonStates import daemonState
from pueue.subcommands.queueDisplaying import executeStatus
from pueue.subcommands.queueManipulation import executeAdd, executeRemove, executeSwitch


class DaemonTesting(unittest.TestCase):
    def setUp(self):
        process = subprocess.Popen(
                'pueue --daemon',
                shell=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
        )
        output, error = process.communicate()

    def tearDown(self):
        args = {}
        daemonState('STOPDAEMON')(args)

    def sendCommand(self, command):
        client =  getClientSocket()
        client.send(pickle.dumps(command, -1))
        answer = client.recv(8192)
        response = pickle.loads(answer)
        client.close()
        return response

    def test_add(self):
        response = self.sendCommand({'mode':'add', 'command': 'ls', 'path': '/tmp'})
        self.assertEqual(response['status'],'success')

    def test_remove(self):
        executeAdd({'command': 'ls'})
        client =  getClientSocket()
        command = {'mode':'remove', 'index': '0'}
        client.send(pickle.dumps(command, -1))
        answer = client.recv(8192)
        response = pickle.loads(answer)
        self.assertEqual(response['status'],'success')
        client.close()
