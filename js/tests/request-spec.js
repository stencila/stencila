describe("Request", function() {

    beforeEach(function() {
      jasmine.Ajax.install();
    });

    afterEach(function() {
      jasmine.Ajax.uninstall();
    });

	it("has a `get` function for getting JSON data", function() {
		jasmine.Ajax.stubRequest('/the/answer/to/life/the/universe/and/everything').andReturn({
			responseText: '{"answer": 42}'
		});
		var deepThought;
		Stencila.Request.get(
			'/the/answer/to/life/the/universe/and/everything',{},
			function(data){
				deepThought = data;
			}
		);
		expect(deepThought.answer).toEqual(42);
	});

	it("has a `post` function for posting JSON data", function() {
		jasmine.Ajax.stubRequest('/hello/world').andReturn({
			responseText: '{"response": "Hey there!"}'
		});
		var deepThought;
		Stencila.Request.post(
			'/hello/world',{},
			{message:"Hello world!"},
			function(data){
				response = data;
			}
		);
		expect(response.response).toEqual("Hey there!");
	});

});