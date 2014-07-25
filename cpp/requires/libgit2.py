# Stencila Waf module for [libgit2](http://libgit2.github.com/)

version = '0.20.0'

def configure(context):
	context.resource(
		'https://github.com/libgit2/libgit2/archive/v%s.zip'%version,
		'libgit2-%s.zip'%version
	)

def build(context):
	zipped = context.resources.make_node('libgit2-%s.zip'%version)
	working = context.path.get_bld()
	unzipped = working.make_node('libgit2')
	compiled = working.make_node('libgit2/build/libgit2.a')

	context(
		cwd = working.abspath(),
		rule = '''
			unzip -qo ${SRC[0].abspath()}
			rm -rf libgit2
			mv libgit2-%s libgit2
		'''%version,
		source = zipped,
		target = unzipped
	)

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
		rule = '''
			ln -sfT ../libgit2/include/git2.h ${TGT[0].abspath()}
			ln -sfT ../libgit2/include/git2   ${TGT[1].abspath()}
		''',
		source = unzipped,
		target = [
			working.make_node('include/git2.h'),
			working.make_node('include/git2')
		]
	)	

	context(
		rule = 'ln -sfT ../libgit2/build/libgit2.a ${TGT}',
		source = compiled,
		target = working.make_node('lib/libgit2.a')
	)	
