describe("Node", function() {

	it("can be constructed from HTML", function() {
		var node = new Stencila.Node('<p>Hello world</p>');

		expect(node.text()).toEqual('Hello world');
	});

	it("can be constructed from a CSS selector", function() {
		var node = new Stencila.Node('#used-in-tests');

		expect(node.text()).toEqual('Test text');
	});

	it("can be constructed from a DOM element", function() {
		var node = new Stencila.Node(document.getElementById('used-in-tests'));

		expect(node.text()).toEqual('Test text');
	});

	it("has `next` and `previous` methods", function() {
		var node = new Stencila.Node(
			'<div>' +
				'<div id="a"></div>' +
				'<div id="b"></div>' +
			'</div>'
		);

		expect(node.select('#a').next().attr('id')).toEqual('b');
		expect(node.select('#b').previous().attr('id')).toEqual('a');
	});

});