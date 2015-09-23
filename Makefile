.PHONY: default, build

default: global-install

global-install:
	sudo python setup.py install
