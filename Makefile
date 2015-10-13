.PHONY: default, dev-install, upload

default: dev-install

dev-install:
	sudo python setup.py develop

clean:
	sudo rm -rf dist
	sudo rm -rf build
	sudo rm -rf pueue.egg-info

dist:
	sudo python setup.py sdist --formats=gztar,zip

upload: clean dist
	twine upload dist/*
