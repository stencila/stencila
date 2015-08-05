describe("Stencil", function() {

    beforeEach(function() {
      jasmine.Ajax.install();
    });

    afterEach(function() {
      jasmine.Ajax.uninstall();
    });

	it("renders a `text` directive", function() {
		var stencil = new Stencila.Stencil('html://'+
			'<span id="a" data-text="answer"></span>' +
			'<span id="b" data-text="answer*2"></span>'
		,{
			answer:42
		});
		stencil.render();

		expect(stencil.select('#a').text()).toEqual('42');
		expect(stencil.select('#b').text()).toEqual('84');
	});

	it("renders an `if` directive", function() {
		stencil = new Stencila.Stencil('html://'+
			'<span id="a" data-if="1"></span>' +
			'<span id="b" data-if="0"></span>'
		,{});
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
		stencil = new Stencila.Stencil(undefined,{answer:42});
		stencil.render();

		expect(stencil.select('#a').text()).toEqual('42');
		expect(stencil.select('#b').attr('data-off')).toEqual('true');
	});

	it("has an `xpath` method", function() {
		var stencil = new Stencila.Stencil('html://'+
			'<div id="a">' +
				'<div id="a1"></div>' + 
			'</div>' +
			'<div id="b"><div><i /><p id="b1"/><i id="b2"/><div></div>'
		);

		expect(stencil.xpath(stencil.select('#a'))).toEqual('/div');
		expect(stencil.xpath(stencil.select('#a1'))).toEqual('/div/div');
		expect(stencil.xpath(stencil.select('#b'))).toEqual('/div[2]');
		expect(stencil.xpath(stencil.select('#b1'))).toEqual('/div[2]/div/p');
		expect(stencil.xpath(stencil.select('#b2'))).toEqual('/div[2]/div/i[2]');
	});

});