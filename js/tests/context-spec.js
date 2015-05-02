describe("Context", function() {

	var context = new Stencila.Context();

	describe("the `assign()` method", function() {

		it("should evaluate expressions and assign them to a variable", function() {
			context.assign('answer','6*7');

			expect(context.get_('answer')).toEqual(42);
		});

	});

});
