describe("Stencil", function() {

    beforeEach(function() {
      jasmine.Ajax.install();
    });

    afterEach(function() {
      jasmine.Ajax.uninstall();
    });

	it("renders a `text` directive", function() {
		var stencil = new Stencila.Stencil(
			'<span id="a" data-text="answer"></span>' +
			'<span id="b" data-text="answer*2"></span>'
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

		expect(stencil.select('#a').attr('data-off')).not.toBeDefined();
		expect(stencil.select('#b').attr('data-off')).toEqual('true');
	});


	it("by default renders the `#content` element", function() {
		// Add #content (will normally be done when the stencil is exported to HTML)
		document.body.innerHTML +=
			'<main id="content" style="display:none">' +
				'<span id="a" data-text="answer"></span>' +
				'<span id="b" data-if="answer==41"></span>' +
			'</main>';
		// Construct Stencil with no arguments
		stencil = new Stencila.Stencil();
		stencil.render({answer:42});

		expect(stencil.select('#a').text()).toEqual('42');
		expect(stencil.select('#b').attr('data-off')).toEqual('true');
	});

});