describe("get", function() {

    beforeEach(function() {
      jasmine.Ajax.install();
    });

    afterEach(function() {
      jasmine.Ajax.uninstall();
    });

	it("will GET a URL", function() {
		jasmine.Ajax.stubRequest('/the/answer/to/life/the/universe/and/everything').andReturn({
			'responseText': '{' +
				' "answer": 42 ' +
			'}'
		});
		var deepThought;
		Stencila.get('/the/answer/to/life/the/universe/and/everything',function(data){
			deepThought = data;
		});
		expect(deepThought.answer).toEqual(42);
	});

});