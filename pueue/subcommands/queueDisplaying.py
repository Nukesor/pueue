import pickle

from pueue.helper.paths import createDir
from pueue.helper.socket import getClientSocket


def executeShow(args):
    client = getClientSocket()
#    if hasattr(args, 'index') and args.index is not None:
#        instruction = {'mode': 'show', 'index': args.index}
#    else:
    instruction = {'mode': 'show', 'index': 'all'}

    # Send new instruction to daemon
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = client.recv(8192)
    answer = pickle.loads(response)
    print('Daemon: {}, Process status: {}, Returncode: {} \n'.format(answer['status'], answer['process'], answer['current']))
    data = answer['data']
    client.close()
    if isinstance(data, str):
        print(data)
    if isinstance(data, list):
        print('Output of command in line: {}'.format(args.index))
        for line in data:
            print(line)
    elif isinstance(data, dict):
        for key, entry in data.items():
            print('Command  #{}:'.format(key))
            print('    {}'.format(entry['command']))
            print('Path: {}\n'.format(entry['path']))


def executeLog(args):
    logPath = createDir() + '/queue.log'
    logFile = open(logPath, 'r')
    print(logFile.read())
