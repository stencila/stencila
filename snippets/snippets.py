'''
A tool for building Stencila snippets

- validation of a snippet against schema
- uploading of a snippet
'''

import os
import argparse
import json

import jsonschema
import requests

schema = json.load(open('schema.json'))

def schema_yaml():
	'''
	Creates a yaml version of the schema for insertion into
	the Stencila API Swagger specification https://stenci.la/api/api.yml
	'''
	import yaml
	yml = yaml.safe_dump(
		schema,
		indent=4,
		default_flow_style=False, 
		encoding='utf-8'
	)
	open('schema.yml','w').write(yml)

def snippet(filename):
	'''
	Load a snippet from a file
	'''
	return json.load(open(os.path.join('snippets', filename)+'.json'))

snippets = [
	snippet('sum'),
	snippet('scatterplot'),
]

def validate():
	'''
	Validate snippets against the schema
	'''
	for snippet in snippets:
		jsonschema.validate(snippet, schema)

def upload():
	'''
	Validate and upload snippets to the Stencila Hub
	'''
	validate()
	token = os.environ.get('STENCILA_HUB_TOKEN')
	if token is None:
		raise Exception('The environment variable STENCILA_HUB_TOKEN must be set')
	for snippet in snippets:
		print 'Uploading', snippet['id'],
		response = requests.put(
			'https://stenci.la/snippets/%s' % snippet['id'], 
			json=snippet,
			auth=("Token", token)
		)
		response.raise_for_status()
		print

parser = argparse.ArgumentParser(description='Validate and upload snippets to the Stencila hub')
parser.add_argument('task')
args = parser.parse_args()

if args.task=='validate':
	validate()
elif args.task=='upload':
	upload()
else:
	print 'Unknown task:', args.task
