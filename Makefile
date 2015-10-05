.PHONY: default, dev-install, upload

default: dev-install

dev-install:
	sudo python setup.py develop

dist:
	sudo python setup.py sdist --formats=gztar,zip

upload: dist
	twine upload dist/*
