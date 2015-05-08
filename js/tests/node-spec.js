describe("Node", function() {

	it("has `next` and `previous` methods", function() {
		var node = new Stencila.Node(
			'<div id="a"></div>' +
			'<div id="b"></div>'
		);

		expect(node.select('#a').next().attr('id')).toEqual('b');
		expect(node.select('#b').previous().attr('id')).toEqual('a');
	});

});