import os
import configparser

from pueue.helper.files import createDir


def getConfig():
    configFile = createDir() + '/pueue.ini'
    config = configparser.ConfigParser()
    # Try to get config, if this doesn't work a new default config will be created
    if os.path.exists(configFile):
        try:
            config.read(configFile)
            return config
        except:
            print('Error while parsing config file. Deleting old config')

    config['default'] = {
        'stopAtError': True,
        'resumeAfterStart': False
    }
    with open(configFile, 'w') as fileDescriptor:
        config.write(fileDescriptor)

    return config
