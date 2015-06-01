describe("Context", function() {

	var context = new Stencila.Context();

	describe("the `execute()` method", function() {

		it("should execute in the global environment", function() {
			context.execute('var j = 0;');
			expect(context.write('j')).toEqual('0');
			context.enter();
			context.assign('j','1');
			expect(context.write('j')).toEqual('1');
			context.exit();
			expect(context.write('j')).toEqual('0');
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
			expect(context.match('"hello"')).not.toBeTruthy();

			context.unmark();
			expect(context.match('42')).not.toBeTruthy();

		});

	});

	describe("the `begin` and `next` methods", function() {

		it("should do nothing with empty lists or objects", function() {
			expect(context.begin('item',[])).not.toBeTruthy();
			expect(context.begin('item',{})).not.toBeTruthy();
		});

		it("should work on lists", function() {
			expect(context.begin('item',[1,2,3])).toBeTruthy();
			expect(context.get_('item')).toEqual(1);
			expect(context.next()).toBeTruthy();
			expect(context.get_('item')).toEqual(2);
			expect(context.next()).toBeTruthy();
			expect(context.get_('item')).toEqual(3);
			expect(context.next()).not.toBeTruthy();
			expect(context.get_('item')).not.toBeDefined();
		});

		it("should work with nested loops", function() {
			expect(context.begin('outer',[1,2,3])).toBeTruthy();
				expect(context.begin('inner',["a","b"])).toBeTruthy();
					
					expect(context.get_('outer')).toEqual(1);
					expect(context.get_('inner')).toEqual("a");
					
					expect(context.next()).toBeTruthy();
					expect(context.get_('outer')).toEqual(1);
					expect(context.get_('inner')).toEqual("b");
					
					expect(context.next()).not.toBeTruthy();
				expect(context.next()).toBeTruthy();
				expect(context.get_('outer')).toEqual(2);
		});

	});

	describe("the `enter` and `exit` methods", function() {

		it("should work as expected ...", function() {
			context.assign('a','10');
			context.enter({a1:11,b1:12});

			expect(context.get_('a')).toEqual(10);
			expect(context.get_('a1')).toEqual(11);
			expect(context.get_('b1')).toEqual(12);

			context.exit();
			
			expect(context.get_('a')).toEqual(10);
			expect(context.get_('a1')).not.toBeDefined();
			expect(context.get_('b1')).not.toBeDefined();
		});

	});

});
