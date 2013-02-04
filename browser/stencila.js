/*!
* Stencila
* 
* Define a Stencila (module)[http://briancray.com/posts/javascript-module-pattern)
*/
var Stencila = (function(){
        
    var Component = Base.extend({
        constructor:function(type,id){
            this.type = type;
            this.id = id;
        },
        get:function(callback){
            $.ajax({
                url:this.type+'/'+this.id,
                dataType:'json',
                context: this
            }).done(function(data){
               callback.call(this,data);
            }).fail(function(jqXHR, textStatus, errorThrown){
                throw 'GET '+this.type+'/'+this.id+' failed:'+textStatus+":"+errorThrown;
            });
        },
    });

    var Stencil = Component.extend({
        constructor:function(id){
            this.base('stencil',id);
        },   
        load:function(){
            this.get(function(data){
                this.data = data;
            });
            return this;
        },
        show:function(){
            var self = this;
            self.view = $(
            '<div class="stencil">\
                <i class="icon-save"></i>\
                <i class="icon-edit"></i>\
                <p class="id"></p>\
                <div class="data">\
                    <p class="status"></p>\
                </div>\
            </div>')
            self.view.on('click','.icon-save',function(event){
                console.log('save');
            });
            self.view.on('click','.icon-edit',function(event){
                self.view.find('.data').attr('contenteditable',true);
                console.log('edit');
            });
            self.view.render(this);
            $('body').append(self.view);
            return self;
        },
        render:function(){
            this.view.render(this);
            return this;
        },
    });
    
    var Theme = Component.extend({
        constructor:function(id){
            this.base('theme',id);
        },   
        // Load the theme into the page by adding a <link> in the <head>
        load:function() {
            if(!this.loaded){
                $('head').append('<link id="'+this.id+'" rel="stylesheet" href="theme/'+this.id+'/css"/>')
                this.loaded = true;
            }
            return this;
        },
        // Unload the theme from the page
        unload:function() {
            if(this.loaded){
                $('head').remove('#'+this.id);
                this.loaded = false;
            }
            return this;
        }
    });
    
    var Dataset = Component.extend({
        constructor:function(id){
            this.base('dataset',id);
        },  
    });
        
    // Classes and function exported by the module
    return {
        Stencil: Stencil,
        Theme: Theme,
        Dataset: Dataset,
    };
    
})();

$(function(){
    // Remove the loading div once everything is, aaahh, loaded.
    $('#loading').remove();
})
