# Stencila Waf module for [WebSocket++](https://github.com/zaphoyd/websocketpp)

version = '0.3.0-alpha4'

def configure(context):
	context.resource(
		'https://github.com/zaphoyd/websocketpp/archive/%s.zip'%version,
		'websocketpp-%s.zip'%version
	)

def build(context):
	context(
		name = 'cpp-requires-websocketpp',
		cwd = context.path.get_bld().abspath(),
		rule = '''
			unzip -qo ${SRC[0].abspath()}
			rm -rf websocket
			mv websocketpp-%s websocket
			ln -sfT ../websocketpp/websocketpp include/websocketpp
		'''%version,
		source = context.resources.make_node('websocketpp-%s.zip'%version),
		target = 'include/websocketpp'
	)
