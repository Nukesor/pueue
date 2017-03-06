from setuptools import setup, find_packages

setup(
    name='pueue',
    author='Arne Beer',
    author_email='arne@twobeer.de',
    version='0.8.0',
    description='Pueue is a fancy queue for shell commands',
    keywords='shell queue command concurrent',
    url='http://github.com/nukesor/pueue',
    license='MIT',
    install_requires=[
        'terminaltables>=3.1.0',
        'daemonize>=2.4.7',
        'colorclass>=2.2.0'
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
