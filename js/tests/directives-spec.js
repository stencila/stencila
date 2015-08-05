describe("Stencil directives", function() {

	// These tests are mainly to check parsing/applying node attributes
	// There is some testing of rendering but it is limited to simple
	// cases. See stencil-spec.js for more complex tests of rendering.

	var context = new Stencila.Context();
	var node;
	beforeEach(function() {
		node = $('<div></div>');
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

	it("include a `text` directive", function() {
		var text = new Stencila.Text('2*2');
		text.set(node);
		text.render(node,context);

		expect(text.expr).toEqual('2*2');

		expect(node.attr('data-text')).toEqual('2*2');
		expect(node.text()).toEqual('4');
	});

	it("include an `if` directive", function() {
		var iff = new Stencila.If('0>1');
		iff.set(node);
		iff.render(node,context);

		expect(iff.expr).toEqual('0>1');

		expect(node.attr('data-if')).toEqual('0>1');
		expect(node.attr('data-off')).toEqual('true');

		var n = $(
			'<div>'+
				'<div id="a" data-if="1"></div>' +
				'<div id="b" data-elif="0"></div>' +
				'<div id="c" data-elif="0"></div>' +
				'<div id="d" data-else=""></div>' +
			'</div>'
		);
		Stencila.directiveRender(n,context);
		expect(n.find('#a').attr('data-off')).not.toBeDefined();
		expect(n.find('#b').attr('data-off')).toEqual('true');
		expect(n.find('#c').attr('data-off')).toEqual('true');
		expect(n.find('#d').attr('data-off')).toEqual('true');

		n.find('#a').attr('data-if','0');
		Stencila.directiveRender(n,context);
		expect(n.find('#a').attr('data-off')).toEqual('true');
		expect(n.find('#b').attr('data-off')).toEqual('true');
		expect(n.find('#c').attr('data-off')).toEqual('true');
		expect(n.find('#d').attr('data-off')).not.toBeDefined();

		n.find('#b').attr('data-elif','1');
		n.find('#c').attr('data-elif','1');
		Stencila.directiveRender(n,context);
		expect(n.find('#a').attr('data-off')).toEqual('true');
		expect(n.find('#b').attr('data-off')).not.toBeDefined();
		expect(n.find('#c').attr('data-off')).toEqual('true');
		expect(n.find('#d').attr('data-off')).toEqual('true');
		
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

	describe("`when` directive", function() {

		it("by default re-renders element when a signal is fired", function() {
			var scope = {
				height: 0.98
			};
			var elem = $(
				'<div data-when="\'height:changed\'">' +
					'<span class="height" data-text="height"></span>' + 
				'</div>'
			);
			var when = new Stencila.When().apply(elem,new Stencila.Context(scope));

			expect(when.then).toEqual('render');

			scope.height = 1.34;
			$(document).trigger('height:changed');
			expect(elem.find('.height').text()).toEqual('1.34');
		});

		it("can delete element when a signal is fired", function() {
			var elem = $(
				// The `when` needs to be nested in this fragment otherwise the 
				// `remove()` won't work
				'<div>'+
					'<div data-when="\'test:remove\' then delete">'+
					'</div>'+
				'</div>'
			);
			var when = new Stencila.When().apply(elem.find('[data-when]'),context);

			expect(when.then).toEqual('delete');

			expect(elem.find('[data-when]').length).toEqual(1);
			$(document).trigger('test:remove');
			expect(elem.find('[data-when]').length).toEqual(0);
		});

	});

	describe("`react` directive", function() {

		it("has `on` child directives which execute code in a closure",function(){
			// Different types of variables to go into context
			var num = 0;
			var object = {x: 0};
			var array = [0];
			var context = new Stencila.Context({
				num: num,
				object: object,
				array: array
			});

			node.html(
				'<button data-react="true">' +
					'<div data-on="click">' +
						'num = num + 1;' +
						'object.x = object.x + 1;' +
						'array[0] = array[0] + 1;' +
					'</div>' +
				'</button>'
			);
			var react = new Stencila.React().apply(node,context);

			// Note that the plain number (a primitive type) is copied by value
			// not by reference in the capture so it does not get updated.
			// See http://stackoverflow.com/questions/10231868/pointers-in-javascript
			node.find('button').click();
			expect(num).toEqual(0);
			expect(object.x).toEqual(1);
			expect(array[0]).toEqual(1);

			node.find('button').click();
			expect(num).toEqual(0);
			expect(object.x).toEqual(2);
			expect(array[0]).toEqual(2);
		});

	});

	describe("`click` directive", function() {

		it("is a shorthand for `react`+`on=click`",function(){
			var module = (function(module){
				module.x = 0;
				module.increment = function(){
					module.x = module.x + 1;
				}
				return module;
			})({});
			node.html('<button data-click="module.increment()"></button>');
			var click = new Stencila.Click().apply(node,new Stencila.Context({module:module}));

			node.find('button').click();
			expect(module.x).toEqual(1);
			
			node.find('button').click();
			expect(module.x).toEqual(2);
		});

	});

});