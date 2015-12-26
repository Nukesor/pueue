import os
import pickle
import unittest
import subprocess

from pueue.helper.socket import getClientSocket
from pueue.helper.files import createDir


class DaemonTesting(unittest.TestCase):
    def setUp(self):
        queue = createDir()+'/queue'
        if os.path.exists(queue):
            os.remove(queue)

        process = subprocess.Popen(
            'pueue --daemon',
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        output, error = process.communicate()
        self.commandFactory('reset')

    def tearDown(self):
        self.commandFactory('STOPDAEMON')

    def commandFactory(self, state):
        instruction = {'mode': state}
        return self.sendCommand(instruction)

    def sendCommand(self, command):
        client = getClientSocket()
        client.send(pickle.dumps(command, -1))
        answer = client.recv(8192)
        response = pickle.loads(answer)
        client.close()
        return response

    def executeAdd(self, command):
        command['mode'] = 'add'
        command['status'] = 'queued'
        command['returncode'] = ''
        command['path'] = '/tmp'
        self.sendCommand(command)

    def executeSwitch(self, command):
        command['mode'] = 'switch'
        self.sendCommand(command)

    def getStatus(self):
        status = self.sendCommand({'mode': 'status'})
        return status

    def test_pause(self):
        status = self.getStatus()
        self.assertEqual(status['status'], 'running')
        self.commandFactory('pause')
        status = self.getStatus()
        self.assertEqual(status['status'], 'paused')

    def test_start(self):
        self.commandFactory('pause')
        self.commandFactory('start')
        status = self.getStatus()
        self.assertEqual(status['status'], 'running')

    def test_add(self):
        self.commandFactory('pause')
        response = self.sendCommand({
            'mode': 'add',
            'command': 'ls',
            'path': '/tmp',
            'status': 'queued',
            'returncode': ''
        })
        self.assertEqual(response['status'], 'success')
        status = self.getStatus()
        self.assertEqual(status['data'][0]['command'], 'ls')
        self.assertEqual(status['data'][0]['path'], '/tmp')

    def test_remove_fails(self):
        response = self.sendCommand({'mode': 'remove', 'key': 0})
        self.assertEqual(response['status'], 'error')

    def test_remove_running(self):
        self.executeAdd({'command': 'sleep 60'})
        response = self.sendCommand({'mode': 'remove', 'key': 0})
        self.assertEqual(response['status'], 'error')

    def test_remove(self):
        self.commandFactory('pause')
        status = self.getStatus()
        self.assertEqual(status['status'], 'paused')
        self.executeAdd({'command': 'ls'})

        response = self.sendCommand({'mode': 'remove', 'key': 0})
        self.assertEqual(response['status'], 'success')
        status = self.getStatus()
        self.assertFalse('0' in status['data'])

    def test_switch(self):
        self.commandFactory('pause')
        self.executeAdd({'command': 'ls'})
        self.executeAdd({'command': 'ls -l'})
        self.executeSwitch({'first': 0, 'second': 1})
        status = self.getStatus()
        self.assertEqual(status['data'][0]['command'], 'ls -l')
        self.assertEqual(status['data'][1]['command'], 'ls')

    def test_switch_fails(self):
        response = self.sendCommand({'mode': 'switch', 'first': 0, 'second': 1})
        self.assertEqual(response['status'], 'error')

    def test_switch_running(self):
        self.executeAdd({'command': 'sleep 60'})
        self.executeAdd({'command': 'ls -l'})
        response = self.sendCommand({
            'mode': 'switch',
            'first': 0,
            'second': 1
        })
        self.assertEqual(response['status'], 'error')

    def test_kill(self):
        self.executeAdd({'command': 'sleep 60'})
        self.commandFactory('kill')
        status = self.getStatus()
        self.assertEqual(status['status'], 'paused')
        self.assertEqual(status['process'], 'No running process')

    def test_stop(self):
        self.executeAdd({'command': 'sleep 60'})
        self.commandFactory('stop')
        status = self.getStatus()
        self.assertEqual(status['status'], 'paused')
        self.assertEqual(status['process'], 'No running process')

    def test_status(self):
        self.executeAdd({'command': 'sleep 60'})
        status = self.getStatus()
        self.assertEqual(status['status'], 'running')
        self.assertEqual(status['process'], 'running')

    def test_reset_paused(self):
        self.commandFactory('pause')
        self.executeAdd({'command': 'sleep 60'})
        self.executeAdd({'command': 'sleep 60'})
        self.commandFactory('reset')
        status = self.getStatus()
        self.assertEqual(status['status'], 'paused')
        self.assertEqual(status['data'], 'Queue is empty')

    def test_reset_running(self):
        self.commandFactory('start')
        self.executeAdd({'command': 'sleep 60'})
        self.executeAdd({'command': 'sleep 60'})
        self.commandFactory('reset')
        status = self.getStatus()
        self.assertEqual(status['status'], 'running')
        self.assertEqual(status['data'], 'Queue is empty')
