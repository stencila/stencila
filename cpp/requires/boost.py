# Stencila Waf module for [Boost C++ libraries](http://www.boost.org/)

version = '1_55_0'

def configure(context):
	context.resource(
		'http://prdownloads.sourceforge.net/boost/boost_%s.tar.bz2'%version,
		'boost_%s.tar.bz2'%version
	)

def build(context):
	zipped = context.resources.make_node('boost_%s.tar.bz2'%version)
	working = context.path.get_bld()
	unzipped = working.make_node('boost')
	#compiled = working.make_node('boost')

	context(
		cwd = working.abspath(),
		rule = '''
			tar --bzip2 -xf ${SRC[0].abspath()}
			rm -rf boost
			mv boost_%s boost
		'''%version,
		source = zipped,
		target = unzipped
	)

	context(
		rule = 'ln -sfT ../boost/include/boost ${TGT[0].abspath()}',
		source = unzipped,
		target =  working.make_node('include/boost')
	)	

if 0:

	context(
		cwd = working.abspath(),
		rule = '''
			cd libgit2
			mkdir -p build
			cd build
			cmake .. -DCMAKE_C_FLAGS=-fPIC -DBUILD_SHARED_LIBS=OFF
			cmake --build .
		''',
		source = unzipped,
		target = compiled
	)

	context(
		rule = 'ln -sfT ../libgit2/build/libgit2.a ${TGT}',
		source = compiled,
		target = working.make_node('lib/libgit2.a')
	)	
