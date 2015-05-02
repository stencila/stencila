describe("Context", function() {

	var context = new Stencila.Context();

	describe("the `assign()` method", function() {

		it("should evaluate expressions and assign them to a variable", function() {
			context.assign('answer','6*7');

			expect(context.get_('answer')).toEqual(42);
		});

		it("should assign to the top of the scope stack", function() {
			context.assign('a','42');
			context.push_();

			context.assign('b','21');
			expect(context.get_('a')).not.toBeDefined();
			expect(context.get_('b')).toEqual(21);

			context.pop_();
			expect(context.get_('a')).toEqual(42);
			expect(context.get_('b')).not.toBeDefined();
		});

	});

	describe("the `write()` method", function() {

		it("should return a string of an expression", function() {
			context.assign('x','42');
			context.assign('y','3.14');
			context.assign('z','"name"');

			expect(context.write('x+1')).toEqual('43');
			expect(context.write('y')).toEqual('3.14');
			expect(context.write('z+" is joe"')).toEqual('name is joe');
		});

	});

	describe("the `test()` method", function() {

		it("should return true/false for an expression", function() {
			expect(context.test('1')).toBeTruthy();
			expect(context.test('0')).not.toBeTruthy();
			expect(context.test('1.27653<2.026536')).toBeTruthy();
		});

	});

	describe("the `mark`,`match` and `unmark` methods", function() {

		it("should work as expected ...", function() {
			context.mark('6*7');

			expect(context.match('42')).toBeTruthy();
			expect(context.match('7*6')).toBeTruthy();
			expect(context.match('hello')).not.toBeTruthy();

			context.unmark();
			expect(function(){
				context.match('42');
			}).toRaise();

		});

	});

});
