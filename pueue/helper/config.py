import os
import configparser


def get_config(config_dir):
    configFile = os.path.join(config_dir, 'pueue.ini')
    config = configparser.ConfigParser()
    # Try to get configuration file and return it
    # If this doesn't work, a new default config file will be created
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
    config['log'] = {
        'logTime': 60*60*24*14,
    }
    with open(configFile, 'w') as fileDescriptor:
        config.write(fileDescriptor)

    return config
