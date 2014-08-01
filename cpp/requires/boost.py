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

	libraries = ['filesystem','python','regex','system','unit_test_framework']

	# Boost is built with some options to override defaults
	#   --with-libraries so that only those libraries that are needed are built
	#  	--prefix=.  - so that boost installs into its own directory  (boost_$(BOOST_VERSION))
	#  	cxxflags=-fPIC - so that the statically compiled library has position independent code for use in shared libraries
	# 	link=static - so that get statically compiled instead of dynamically compiled libraries
	# 	
	# TODO
	#   Need to add the building of libboost_python3.a. This gets built if we add the lines
	#		# Python configuration
	#		using python : 2.6 ;
	#		using python : 2.7 ;
	#		using python : 3.2 ;
	#   to the project-config.jam.
	#   Should use context.env.PYTHON_VERSIONS to do this
	#   See http://www.boost.org/doc/libs/1_55_0/libs/python/doc/building.html#id34
	#   
	#   An alternative may to be to not use a project-config.jam and instead use a hand coded user-config.jam
	#   based on one that bootstrap.sh produces.
	if context.env.SYSTEM=='linux':
		boost_build = '''
			cd boost
			./bootstrap.sh --with-libraries=%(libraries)s
			./b2 --prefix=. cxxflags=-fPIC link=static install
		'''
	elif context.env.SYSTEM=='msys':
		# Under MSYS some differences are required
		#	- bootstrap.sh must be called with mingw specified as toolset otherwise errors occur
		#	- project-config.jam must be edited to fix the [error](http://stackoverflow.com/a/5244844/1583041) produced by the above command
		#	- b2 must be called with "system" layout of library names and header locations (otherwise it defaults to 'versioned' on Windows)
		#	- b2 must be called with "release" build otherwise defaults to debug AND release, which with "system" causes an error (http://boost.2283326.n4.nabble.com/atomic-building-with-layout-system-mingw-bug-7482-td4640920.html)
		boost_build = '''
			cd boost
			./bootstrap.sh --with-libraries=%(libraries)s --with-toolset=mingw;
			sed -i "s/mingw/gcc/g" project-config.jam;
			./b2 --prefix=. --layout=system release toolset=gcc cxxflags=-fPIC link=static install
		'''
	context(
		cwd = working.abspath(),
		rule = boost_build%{
			'libraries' : ','.join(libraries).replace('unit_test_framework','test')
		},
		source = unzipped,
		# Targets are include directory and each of the libraires
		target = [working.make_node('boost/include/boost')] + 
				 [working.make_node('boost/lib/libboost_%s.a'%library) for library in libraries] +
				 [working.make_node('boost/lib/libboost_python3.a')]
	)

	# Create a symbolic link in `include`
	context(
		rule = 'ln -sfT ../boost/include/boost ${TGT[0].abspath()}',
		source = working.make_node('boost/include/boost'),
		target = working.make_node('include/boost')
	)

	# Create a symbolic links in `lib`
	for library in libraries + ['python3']:
		context(
			rule = 'ln -sfT ../boost/lib/libboost_%s.a ${TGT[0].abspath()}'%library,
			source = 'boost/lib/libboost_%s.a'%library,
			target =  'lib/libboost_%s.a'%library
		)
