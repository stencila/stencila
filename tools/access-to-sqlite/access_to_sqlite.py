#!/usr/bin/env python

# Converts a Microsoft Access database into an SQLite database
#
#
# Uses mdbtools (http://mdbtools.sourceforge.net/).
# Your operating system may have a binary package for mbdtools. e.g. Ubuntu
#
#	sudo apt-get install mdbtools
#
# Requires python package `sh`:
#
#	sudo pip install sh

import re
import sqlite3
import sh


def schema(source,dest=None):
	'''
	Export the database schema to an SQL file
	'''

	# Obtain schema
	sql = str(sh.Command('mdb-schema')(source,'mysql'))

	# Comment out SQL that is not supported by SQLite
	sql = re.sub(
		'COMMENT ON',
		'-- COMMENT ON',
		sql
	)

	# Write to file?
	if dest: file(dest,'w').write(sql)

	return sql

def tables(source):
	'''
	Obtain a list of tables
	'''

	return sh.Command('mdb-tables')(source).strip().split()

def export(source,table,dest=None):
	'''
	Export a table as a series of SQL insert statements.
	Exports to a file, optionally a temporary one, so that large files can be handled.
	'''

	sh.Command('mdb-export')(source,table,
		I = 'mysql', #Export as SQL INSERT statements instead of CSV
		_out = dest
	)


def create(sql,dest):
	'''
	Create a new SQLite database using a SQL schema
	'''
	# Run schema SQL (by piping via echo to sqlite)
	sh.sqlite3(sh.echo(sql),dest)

def load(source,table,dest):
	'''
	Load a table into an SQLite database
	'''
	filename = 'temp-%s.sql'%table
	# Export table
	export(source,table,filename)
	# Wrap in a transaction
	sqlite = sqlite3.connect(dest,isolation_level="EXCLUSIVE")
	sqlite.execute('BEGIN;')
	sqlite.executescript(file(filename).read())
	sqlite.execute('END;')
	sqlite.commit()

def load_all(source,dest):
	'''
	Load all tables into an SQLite database
	'''
	for table in tables(source):
		load(source,table,dest)

def convert(source,dest):
	'''
	Convert Access database to an SQLite database
	'''
	# Create
	create(schema(source),dest)
	# Load all tables
	load_all(source,dest)

