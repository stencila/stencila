describe("Stencil", function() {

    beforeEach(function() {
      jasmine.Ajax.install();
    });

    afterEach(function() {
      jasmine.Ajax.uninstall();
    });

	it("renders a `write` directive", function() {
		var stencil = new Stencila.Stencil(
			'<span id="a" data-write="answer"></span>' +
			'<span id="b" data-write="answer*2"></span>'
		);
		stencil.render({
			answer:42
		});

		expect(stencil.select('#a').text()).toEqual('42');
		expect(stencil.select('#b').text()).toEqual('84');
	});

	it("renders an `if` directive", function() {
		stencil = new Stencila.Stencil(
			'<span id="a" data-if="1"></span>' +
			'<span id="b" data-if="0"></span>'
		);
		stencil.render();

		expect(stencil.select('#a').attr('data-off')).toEqual('');
		expect(stencil.select('#b').attr('data-off')).toEqual('true');
	});


	it("renders a page with a context loaded from a GET request", function() {
		// Add #content (will normally be done when the stencil is exported to HTML)
		document.body.innerHTML += 
			'<main id="content" style="display:none">' +
				'<span id="a" data-write="answer"></span>' +
				'<span id="b" data-if="answer==41"></span>' +
			'</main>';
		// Mock a response
		jasmine.Ajax.stubRequest('/same/url').andReturn({
			'responseText': '{' +
				' "answer": 42, ' +
				' "items": [10,11,12] ' +
			'}'
		});
		// Construct with arguments so mocking works (normally the default arguments will be used)
		stencil = new Stencila.Stencil('#content','/same/url');
		stencil.render();

		expect(stencil.select('#a').text()).toEqual('42');
		expect(stencil.select('#b').attr('data-off')).toEqual('true');
	});

});