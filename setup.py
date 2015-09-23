from setuptools import setup, find_packages

setup(
    name='pueue',
    author='Arne Beer',
    author_mail='arne@twobeer.de',
    version='0.0.1',
    description='A bash command queue written in python',
    keywords='bash queue command',
    url='http://github.com/nukesor/pueue',
    license='MIT',
    classifiers=[
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3.4',
        'Programming Language :: Python :: 3.5',
        'Environment :: Console'
    ],
    packages=find_packages('pueue'),
    package_dir={'': 'pueue'},
    entry_points={
            'console_scripts': [
                'pueue=pueue:main'
            ]
    })
