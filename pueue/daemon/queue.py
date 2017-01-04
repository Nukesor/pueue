import os
import pickle


class Queue():
    def __init__(self, daemon):
        self.daemon = daemon
        self.read()
        if len(self.queue) > 0:
            self.nextKey = max(self.queue.keys()) + 1
        else:
            self.nextKey = 0

    def keys(self):
        return self.queue.keys()

    def __len__(self):
        return len(self.queue)

    def __getitem__(self, key):
        return self.queue[key]

    def __setitem__(self, key, value):
        self.queue[key] = value

    def __delitem__(self, key):
        del self.queue[key]

    def items(self):
        return self.queue.items()

    def reset(self):
        self.queue = {}
        self.write()

    def next(self):
        # Get the next processable item of the queue.
        # Returns None if no key is found.
        # This is the
        smallest = None
        for key in self.queue.keys():
            if self.queue[key]['status'] == 'queued':
                if smallest is None or key < smallest:
                    smallest = key
                    self.current = self.queue[smallest]
                    self.current_key = smallest
        return smallest

    def read(self):
        queuePath = self.daemon.config_dir+'/queue'
        if os.path.exists(queuePath):
            queueFile = open(queuePath, 'rb')
            try:
                self.queue = pickle.load(queueFile)
            except:
                print('Queue file corrupted, deleting old queue')
                os.remove(queuePath)
                self.queue = {}
            queueFile.close()
        else:
            self.queue = {}

    def write(self):
        queuePath = self.daemon.config_dir + '/queue'
        queueFile = open(queuePath, 'wb+')
        try:
            pickle.dump(self.queue, queueFile, -1)
        except:
            print('Error while writing to queue file. Wrong file permissions?')
        queueFile.close()

    def add_new(self, command):
        self.queue[self.nextKey] = command
        self.queue[self.nextKey]['status'] = 'queued'
        self.queue[self.nextKey]['returncode'] = ''
        self.queue[self.nextKey]['stdout'] = ''
        self.queue[self.nextKey]['stderr'] = ''
        self.queue[self.nextKey]['start'] = ''
        self.queue[self.nextKey]['end'] = ''

        self.nextKey += 1
        self.write()
        return {'message': 'Command added', 'status': 'success'}

    def remove(self, command):
        key = command['key']
        if key not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(key)), 'status': 'error'}
        else:
            # Delete command from queue, save the queue and send response to client
            if not self.daemon.paused and key == self.current_key:
                answer = {
                    'message': "Can't remove currently running process, "
                    "please stop the process before removing it.",
                    'status': 'error'
                }
            else:
                del self.queue[key]
                self.write()
                answer = {'message': 'Command #{} removed'.format(key), 'status': 'success'}
        return answer

    def restart(self, command):
        key = command['key']
        if key not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(key)), 'status': 'error'}
        else:
            # Delete command from queue, save the queue and send response to client
            if self.queue[key]['status'] == 'queued':
                answer = {'message': 'Command #{} is already queued'
                          .format(key), 'status': 'success'}
            if self.queue[key]['status'] in ['running', 'stopping', 'killing']:
                answer = {'message': 'Command #{} is currently running'
                          .format(key), 'status': 'error'}
            else:
                self.queue[self.nextKey] = {}
                self.queue[self.nextKey]['command'] = self.queue[key]['command']
                self.queue[self.nextKey]['path'] = self.queue[key]['path']
                self.queue[self.nextKey]['status'] = 'queued'
                self.queue[self.nextKey]['returncode'] = ''
                self.queue[self.nextKey]['start'] = ''
                self.queue[self.nextKey]['end'] = ''
                self.nextKey += 1
                self.write()
                answer = {'message': 'Command #{} queued again'
                          .format(key), 'status': 'success'}
        return answer

    def switch(self, command):
        first = command['first']
        second = command['second']
        # Send error answer to client in case there exists no such key
        if first not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(first)), 'status': 'error'}
        elif second not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(second)), 'status': 'error'}
        else:
            # Delete command from queue, save the queue and send response to client
            if not self.daemon.paused and (first == self.current_key or second == self.current_key):
                answer = {
                    'message': "Can't switch currently running process, "
                    "please stop the process before switching it.",
                    'status': 'error'
                }
            else:
                tmp = self.queue[second].copy()
                self.queue[second] = self.queue[first].copy()
                self.queue[first] = tmp
                answer = {
                    'message': 'Command #{} and #{} switched'.format(first, second),
                    'status': 'success'
                }
        return answer
