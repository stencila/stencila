# Stencila Waf module for [tidy-html5](http://w3c.github.com/tidy-html5/)

def configure(context):
	context.resource('https://github.com/w3c/tidy-html5/archive/master.zip','tidyhtml5-master.zip')

def build(context): 	
	source = context.path.get_src()
	target = context.path.get_bld()
	zipped = context.resources.make_node('tidyhtml5-master.zip')
	unzipped = target.make_node('tidyhtml5')
	included = target.make_node('include/tidyhtml5')
	libraried = target.make_node('lib/libtidyhtml5.a')

	context(
		rule = 'unzip -qo ${SRC} -d %s; rm -rf ${TGT}; mv %s/tidy-html5-master ${TGT}'%(target.abspath(),target.abspath()),
		source = zipped,
		target = unzipped
	)
	# Apply patch to Makefile to add -O3 -fPIC options
	context(
		name = 'tidyhtml5-patch1',
		rule = 'patch ${TGT[0].abspath()} ${SRC[1].abspath()}',
		source = [unzipped,source.find_node('tidy-html5-build-gmake-Makefile.patch')],
		target = target.make_node('tidyhtml5/build/gmake/Makefile')
	)
	# Apply patch from pull request #98 (https://github.com/w3c/tidy-html5/pull/98.patch) to add <main> tag 
	# (this is applied using `patch` rather than `git` so that `git` is not required) 
	context(
		name = 'tidyhtml5-patch2',
		cwd = unzipped.abspath(),
		rule = 'patch -p1 -i ${SRC[1].abspath()}',
		source = [unzipped,source.find_node('tidy-html5-pull-98.patch')],
		target = [
			target.make_node('tidyhtml5/include/tidyenum.h'),
			target.make_node('tidyhtml5/src/attrdict.h'),
			target.make_node('tidyhtml5/src/attrdict.c'),
			target.make_node('tidyhtml5/src/tags.c')
		]
	)
	# Apply patch to prevent linker error associated with "GetFileSizeEx" under MSYS
	context(
		name = 'tidyhtml5-patch3',
		rule = 'patch ${TGT[0].abspath()} ${SRC[1].abspath()}',
		source = [unzipped,source.find_node('tidy-html5-src-mappedio.c.patch')],
		target = target.make_node('tidyhtml5/src/mappedio.c')
	)
	# Compile using "make ../../lib/libtidy.a" ("make all" is not required)
	context(
		name = 'tidyhtml5-make',
		cwd = '%s/build/gmake'%unzipped.abspath(),
		rule = 'make ../../lib/libtidy.a',
		after = ['tidyhtml5-patch1','tidyhtml5-patch2','tidyhtml5-patch3']
	)
	context(
		rule = 'ln -sfT ../tidyhtml5/include ${TGT}',
		source = unzipped,
		target = included
	)	
	context(
		rule = 'ln -sfT ../tidyhtml5/lib/libtidy.a ${TGT}',
		after = 'tidyhtml5-make',
		target = libraried
	)	
