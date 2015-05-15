describe("Stencil directives", function() {

	// These tests are mainly fto check parsing/applying node attributes
	// There is some testing of rendering but it is limited to simple
	// cases. See stencil-spec.js for more complex tests of rendering.

	var context = new Stencila.Context();
	var node;
	beforeEach(function() {
		node = new Stencila.Node('<div></div>');
	});

	it("include an `exec` directive", function() {
		var exec = new Stencila.Exec('js','var xyb26f82=24;');
		exec.set(node);
		exec.render(node,context);

		expect(exec.code).toEqual('var xyb26f82=24;');

		expect(node.attr('data-exec')).toEqual('js');
		expect(node.text()).toEqual('var xyb26f82=24;');
		expect(window.xyb26f82).toEqual(24);
	});

	it("include a `write` directive", function() {
		var write = new Stencila.Write('2*2');
		write.set(node);
		write.render(node,context);

		expect(write.expr).toEqual('2*2');

		expect(node.attr('data-write')).toEqual('2*2');
		expect(node.text()).toEqual('4');
	});

	it("include an `if` directive", function() {
		var iff = new Stencila.If('0>1');
		iff.set(node);
		iff.render(node,context);

		expect(iff.expr).toEqual('0>1');

		expect(node.attr('data-if')).toEqual('0>1');
		expect(node.attr('data-off')).toEqual('true');

		var n = new Stencila.Node(
			'<div>'+
				'<div id="a" data-if="1"></div>' +
				'<div id="b" data-elif="0"></div>' +
				'<div id="c" data-elif="0"></div>' +
				'<div id="d" data-else=""></div>' +
			'</div>'
		);
		Stencila.directiveRender(n,context);
		expect(n.select('#a').has('data-off')).not.toBeTruthy();
		expect(n.select('#b').has('data-off')).toBeTruthy();
		expect(n.select('#c').has('data-off')).toBeTruthy();
		expect(n.select('#d').has('data-off')).toBeTruthy();

		n.select('#a').attr('data-if','0');
		Stencila.directiveRender(n,context);
		expect(n.select('#a').has('data-off')).toBeTruthy();
		expect(n.select('#b').has('data-off')).toBeTruthy();
		expect(n.select('#c').has('data-off')).toBeTruthy();
		expect(n.select('#d').has('data-off')).not.toBeTruthy();

		n.select('#b').attr('data-elif','1');
		n.select('#c').attr('data-elif','1');
		Stencila.directiveRender(n,context);
		expect(n.select('#a').has('data-off')).toBeTruthy();
		expect(n.select('#b').has('data-off')).not.toBeTruthy();
		expect(n.select('#c').has('data-off')).toBeTruthy();
		expect(n.select('#d').has('data-off')).toBeTruthy();
		
	});

	it("include a `for` directive", function() {
		node.html('<div data-write="name"></div>');

		var forr = new Stencila.For('name','["Joe","Sally","Jane"]');
		forr.set(node);
		forr.render(node,context);

		expect(forr.item).toEqual('name');
		expect(forr.items).toEqual('["Joe","Sally","Jane"]');

		expect(node.attr('data-for')).toEqual('name in ["Joe","Sally","Jane"]');
		//console.log(node.html());
	});

});