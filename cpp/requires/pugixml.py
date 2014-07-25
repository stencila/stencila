# Stencila Waf module for [pugixml](http://pugixml.org/)

version = '1.2'

def configure(context):
	context.resource(
		'http://pugixml.googlecode.com/files/pugixml-%s.tar.gz'%version,
		'pugixml-%s.tar.gz'%version
	)

def build(context):
	target = context.path.get_bld()
	zipped = context.resources.make_node('pugixml-%s.tar.gz'%version)
	unzipped = target.make_node('pugixml')
	compiled = target.make_node('pugixml/src/libpugixml.a')

	context(
		rule = '''
			mkdir -p ${TGT}
			cd ${TGT} 
			tar xzf ${SRC[0].abspath()}
		''',
		source = zipped,
		target = unzipped
	)

	context(
		name = 'pugixml-make',
		rule = '''
			cd ${SRC}/src
			${CXX} -O3 -fPIC -c pugixml.cpp
			${AR} rcs libpugixml.a pugixml.o
		''',
		source = unzipped,
		target = compiled
	)

	context(
		rule = 'ln -sfT ../pugixml/src/pugixml.hpp ${TGT}',
		source = unzipped,
		target = target.make_node('include/pugixml.hpp')
	)	
	context(
		rule = 'ln -sfT ../pugixml/src/pugiconfig.hpp ${TGT}',
		source = unzipped,
		target = target.make_node('include/pugiconfig.hpp')
	)
	context(
		rule = 'ln -sfT ../pugixml/src/libpugixml.a ${TGT}',
		source = compiled,
		target = target.make_node('lib/libpugixml.a')
	)	
