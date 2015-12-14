from setuptools import setup, find_packages

setup(
    name='pueue',
    author='Arne Beer',
    author_email='arne@twobeer.de',
    version='0.4.1',
    description='Pueue is a fancy queue for bash commands',
    keywords='bash queue command',
    url='http://github.com/nukesor/pueue',
    license='MIT',
    install_requires=[
        'terminaltables==2.1.0',
        'daemonize==2.4.1',
        'colorclass==1.2.0'
    ],
    classifiers=[
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3.3',
        'Programming Language :: Python :: 3.4',
        'Programming Language :: Python :: 3.5',
        'Environment :: Console'
    ],
    packages=find_packages(),
    entry_points={
            'console_scripts': [
                'pueue=pueue:main'
            ]
    })
