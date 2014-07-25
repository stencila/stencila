#! /usr/bin/env python
# encoding: utf-8

# Top level Waf wscript file for Stencila library
# See http://docs.waf.googlecode.com/git/book_17/single.html

# Project directory
top = '.'

# Build directory uses a heirarchy based on the 
# operating system and machine architecture.
import os
import platform
out = os.path.join(
	# Top level build directory
	'builds',
	# Operating system e.g. linux
	platform.system().lower(),
	# Instruction set architecture e.g i386, x86_64
	platform.machine().lower()
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
	# Create a `resource` function on context for checking
	# and downloading resources
	def resource(url,filename):
		resource = os.path.join(resources,filename)
		print 'Checking for resource "%s"'%resource
		if not os.path.exists(resource):
			print 'Downloading resource "%s"'%resource
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
