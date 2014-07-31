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

	# Boost is built with some options to override defaults
	#  	--prefix=.  - so that boost installs into its own directory  (boost_$(BOOST_VERSION))
	#  	cxxflags=-fPIC - so that the statically compiled library has position independent code for use in shared libraries
	# 	link=static - so that get statically compiled instead of dynamically compiled libraries
	if context.env.SYSTEM=='linux':
		boost_build = '''
			cd boost
			./bootstrap.sh
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
			./bootstrap.sh --with-toolset=mingw;
			sed -i "s/mingw/gcc/g" project-config.jam;
			./b2 --prefix=. --layout=system release toolset=gcc cxxflags=-fPIC link=static install
		'''
	context(
		cwd = working.abspath(),
		rule = boost_build,
		source = unzipped,
		target = 'boost/lib/libboost_system.a'
	)

	context(
		rule = 'ln -sfT ../boost/include/boost ${TGT[0].abspath()}',
		source = 'boost/lib/libboost_system.a',
		target =  working.make_node('include/boost')
	)
