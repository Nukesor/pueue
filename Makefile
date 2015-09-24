.PHONY: default, uninstall, build

default: dev-install

dev-install:
	sudo python setup.py develop
