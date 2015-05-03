describe("Stencil", function() {

	var context = new Stencila.Context({
		answer:42
	});
	var stencil = new Stencila.Stencil();

	it("renders a `write` directive", function() {
		stencil.html(
			'<span id="a" data-write="answer"></span>' +
			'<span id="b" data-write="answer*2"></span>'
		);
		stencil.render(context);

		expect(stencil.select('#a').text()).toEqual('42');
		expect(stencil.select('#b').text()).toEqual('84');
	});

	it("renders an `if` directive", function() {
		stencil.html(
			'<span id="a" data-if="1"></span>' +
			'<span id="b" data-if="0"></span>'
		);
		stencil.render(context);

		expect(stencil.select('#a').attr('data-off')).toEqual('');
		expect(stencil.select('#b').attr('data-off')).toEqual('true');
	});

});