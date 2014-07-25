# Stencila Waf module for [rapidjson](https://code.google.com/p/rapidjson/)

version = '0.11'

def configure(context):
	# There are several forks of rapidjson on Github
	# At the time of writing the ones that appeared to be most worthwhile watching were:
	# 
	# 	- https://github.com/pah/rapidjson
	# 	- https://github.com/miloyip/rapidjson/issues/1
	# 
	context.resource(
		'http://rapidjson.googlecode.com/files/rapidjson-%s.zip'%version,
		'rapidjson-%s.zip'%version
	)

def build(context): 	
	zipped = context.resources.make_node('rapidjson-%s.zip'%version)
	source = context.path.get_src()
	working = context.path.get_bld()
	unzipped = working.make_node('rapidjson')
	included = working.make_node('include/rapidjson')

	context(
		rule = 'unzip -qo ${SRC} -d ${TGT[0].parent.abspath()}',
		source = zipped,
		target = unzipped
	)
	
	# Apply patch from https://github.com/scanlime/rapidjson/commit/0c69df5ac098640018d9232ae71ed1036c692187
	# that allows for copying of Documents [rapidjson prevents copying 
	# of documents](http://stackoverflow.com/questions/22707814/perform-a-copy-of-document-object-of-rapidjson)
	context(
		cwd = unzipped.abspath()+'/include/rapidjson',
		rule = 'patch -p1 -i ${SRC[1].abspath()}',
		source = [unzipped,source.find_node('rapidjson-scanlime-0c69df5ac0.patch')],
		target = working.make_node('rapidjson/include/rapidjson/document.h')
	)

	context(
		rule = 'ln -sfT ../rapidjson/include/rapidjson ${TGT}',
		source = unzipped,
		target = included
	)	
