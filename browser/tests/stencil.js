describe("A stencil",function(){
    
    var stencil = new Stencila.Stencil("my_id");
    
    it("has an id",function(){
        expect(stencil.id).toEqual("my_id");
    });
    
});
