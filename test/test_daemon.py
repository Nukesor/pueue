import os
import shutil
import unittest
from pueue.daemon.daemon import Daemon

class DaemonTesting(unittest.TestCase):
    def setUp(self):
        home = os.path.expanduser('~')
        queueFolder = home+'/.pueue'
        if os.path.exists(queueFolder):
            shutil.rmtree(queueFolder)
        self.daemon = Daemon()

    def tearDown(self):
        self.daemon.socket.close()


    def test_add(self):
        add_command = {'mode': 'add', 'command': 'ls', 'path': '/usr/lib'}
        self.assertEqual(self.daemon.queue, {})
        self.daemon.executeAdd(add_command)
        self.assertEqual(self.daemon.queue[0], add_command)

    def test_remove(self):
        add_command = {'mode': 'add', 'command': 'ls', 'path': '/usr/lib'}
        self.daemon.executeAdd(add_command)
        self.assertEqual(self.daemon.queue[0], add_command)

        remove_command = {'mode': 'remove', 'key': 0}
        self.daemon.executeRemove(remove_command)
        self.assertEqual(self.daemon.queue, {})

    def test_remove_key_missing(self):
        remove_command = {'mode': 'remove', 'key': 0}
        answer = self.daemon.executeRemove(remove_command)
        self.assertEqual('No command with key #0', answer)

    def test_switch(self):
        first_command= {'mode': 'add', 'command': 'ls', 'path': '/usr/lib'}
        second_command= {'mode': 'add', 'command': 'cd ./', 'path': '/usr'}
        self.daemon.executeAdd(first_command)
        self.daemon.executeAdd(second_command)
        self.assertEqual(self.daemon.queue[0], first_command)
        self.assertEqual(self.daemon.queue[1], second_command)
        switch_command = {'mode': 'switch', 'first': 0, 'second': 1}
        self.daemon.executeSwitch(switch_command)
        self.assertEqual(self.daemon.queue[0], second_command)
        self.assertEqual(self.daemon.queue[1], first_command)

    def test_switch_key_missing(self):
        switch_command = {'mode': 'switch', 'first': 0, 'second': 1}
        answer = self.daemon.executeSwitch(switch_command)
        self.assertEqual(answer, 'No command with key #0')

    def test_reset(self):
        add_command = {'mode': 'add', 'command': 'ls', 'path': '/usr/lib'}
        self.daemon.executeAdd(add_command)
        self.daemon.executeReset()
        self.assertEqual(self.daemon.queue, {})
        self.assertEqual(self.daemon.log, {})
        self.assertFalse(self.daemon.paused)


    def test_start(self):
        self.daemon.executePause()
        self.daemon.executeStart()
        self.assertFalse(self.daemon.paused)

    def test_pause(self):
        self.daemon.executePause()
        self.assertTrue(self.daemon.paused)
