#! /usr/bin/env python
# encoding: utf-8
# 
import os
import platform
import subprocess

# Top level Waf wscript file for Stencila library
# See http://docs.waf.googlecode.com/git/book_17/single.html

# Project directory
top = '.'

# Build directory uses a heirarchy based on the 
# operating system and machine architecture.
system = platform.system().lower()
architecture = platform.machine().lower()

out = os.path.join(
	# Top level build directory
	'builds',
	# Operating system e.g. linux
	system,
	# Instruction set architecture e.g i386, x86_64
	architecture
)

# External dependencies are downloaded to a separate folder
# during configuration. This seems to be better than including
# file download during the build phase because (a) `./waf clean`
# deletes all files (b) the downloaded files can be shared accross build
# variants
resources = 'builds/resources'

def options(context):
	# Recurse into subdirectories
	context.recurse('cpp')
	context.recurse('py')
	context.recurse('r')

def configure(context):
	context.env.SYSTEM = system
	context.env.ARCHIT = architecture

	# Update the Stencila VERSION
	context.env.STENCILA_VERSION = stencila_version()

	# Create a `resource` function on context for checking
	# and downloading resources
	def resource(url,filename):
		resource = os.path.join(resources,filename)
		print('Checking for resource "%s"'%resource)
		if not os.path.exists(resource):
			print('Downloading resource "%s"'%resource)
			os.system('wget --no-check-certificate -O %s %s'%(resource,url))
	context.resource = resource
	# Recurse into subdirectories
	context.recurse('cpp')
	context.recurse('py')
	context.recurse('r')

def build(context):
	# Provide context with a resources Node so that
	# resources can be referred to
	context.resources = context.path.make_node(resources)
	# Recurse into subdirectories
	context.recurse('cpp')
	context.recurse('py')
	context.recurse('r')


def stencila_version():
	# Get Stencila library version number
	from subprocess import Popen, PIPE
	# This uses "--long" so that git produces the same format output each time (even just after a new tag)
	# This uses "--tags" so that lightweight tags (not annotated) are shown in output
	version = Popen("git describe --long --tags", shell=True,stdout=PIPE, stderr=PIPE).communicate()[0].strip()
	# Just get the tag from the front
	version = version[:version.find('-')]
	# Write to file
	version_file = open('VERSION','w')
	version_file.write(version)
	version_file.close()

	print('Setting Stencila version: %s'%version)
	return version