describe("Stencil directives", function() {

	// These tests are mainly fto check parsing/applying node attributes
	// There is some testing of rendering but it is limited to simple
	// cases. See stencil-spec.js for more complex tests of rendering.

	var context = new Stencila.Context();
	var node;
	beforeEach(function() {
    node = new Stencila.Node();
  });

	it("include an `exec` directive", function() {
		var exec = new Stencila.Exec('js','var xyb26f82=24;');
		exec.apply(node);
		exec.render(node,context);

		expect(exec.code).toEqual('var xyb26f82=24;');

		expect(node.attr('data-exec')).toEqual('js');
		expect(node.text()).toEqual('var xyb26f82=24;');
		expect(window.xyb26f82).toEqual(24);
	});

	it("include a `write` directive", function() {
		var write = new Stencila.Write('2*2');
		write.apply(node);
		write.render(node,context);

		expect(write.expression).toEqual('2*2');

		expect(node.attr('data-write')).toEqual('2*2');
		expect(node.text()).toEqual('4');
	});

	it("include an `if` directive", function() {
		var iff = new Stencila.If('0>1');
		iff.apply(node);
		iff.render(node,context);

		expect(iff.expression).toEqual('0>1');

		expect(node.attr('data-if')).toEqual('0>1');
		expect(node.attr('data-off')).toEqual('true');
	});

	it("include a `for` directive", function() {
		node.html('<div data-write="name"></div>');

		var forr = new Stencila.For('name in ["Joe","Sally","Jane"]');
		forr.apply(node);
		forr.render(node,context);

		expect(forr.item).toEqual('name');
		expect(forr.items).toEqual('["Joe","Sally","Jane"]');

		expect(node.attr('data-for')).toEqual('name in ["Joe","Sally","Jane"]');
		console.log(node.html());
	});

});