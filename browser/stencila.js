/*!
* Stencila
* 
* Define a Stencila (module)[http://briancray.com/posts/javascript-module-pattern)
*/
var Stencila = (function(){

    /*
     Sets up an ACE editor instance on the element with the id
    */
    var Editor = function(id,mode){
        var editor = ace.edit(id);
        editor.setTheme("ace/theme/twilight");
        editor.getSession().setMode(new (require("ace/mode/"+mode).Mode)());
        return editor;
    }

    var Component = Base.extend({
        constructor:function(type,id){
            this.type = type;
            this.id = id;
            if(id) this.get();
            else this.post();
        },
        
        post:function(data){
            if(!this.id){
                $.ajax({
                    type:'POST',
                    url:this.type,
                    data:JSON.stringify(data),
                    dataType:'json',
                    async: false,
                    context: this
                }).done(function(data){
                   if(data.error) throw data.error;
                   this.id = data.id;
                }).fail(function(jqXHR, textStatus, errorThrown){
                    throw 'POST '+this.type+' failed:'+textStatus+":"+errorThrown;
                });
            }
            return this;
        },
        
        get:function(callback){
            $.ajax({
                type:'GET',
                url:this.type+'/'+this.id,
                dataType:'json',
                async: false,
                context: this
            }).done(function(data){
               if(data.error) throw data.error;
               callback.call(this,data);
            }).fail(function(jqXHR, textStatus, errorThrown){
                throw 'GET '+this.type+'/'+this.id+' failed:'+textStatus+":"+errorThrown;
            });
            return this;
        },
        
        put:function(data){
            $.ajax({
                type:'PUT',
                url:this.type+'/'+this.id,
                data:JSON.stringify(data),
                dataType:'json',
                context: this
            }).done(function(data){
                if(data.error) throw data.error;
            }).fail(function(jqXHR, textStatus, errorThrown){
                throw 'GET '+this.type+'/'+this.id+' failed:'+textStatus+":"+errorThrown;
            });
            return this;
        },
        
        handle: function(){
            return this.type+'-'+this.id.replace('.','-');
        },
    });

    var Stencil = Component.extend({
        constructor:function(id){
            this.body = "";
            this.theme = null;
            this.base('stencil',id);
        },   
        
        get:function(){
            this.base(function(data){
                this.body = data.body;
            });
            return this;
        },
        
        put: function(){
            this.base({
                body : this.body
            });
            return this;
        },
        
        show:function(){
            var bodyEditorHandle = this.handle()+'-body-editor';

            this.view = $(
            '<div class="stencil">\
                Stencil <span class="id"></span>\
                <i class="icon-save"></i>\
                <i class="icon-edit"></i>\
                <div class="body"></div>\
                <div class="body-editor" id="'+bodyEditorHandle+'"></div>\
            </div>').appendTo('body');

            this.bodyEditor = Editor(bodyEditorHandle,'html');
            
            var self = this;
            this.view.on('click','.icon-save',function(event){
                self.put();
            });
            this.view.on('click','.icon-edit',function(event){
                self.view.find('.body')
                    .attr('contenteditable',true);
                //self.body = self.view.find('.body').html();
            });
            this.bodyEditor.getSession().on('change',function(event){
                self.body = self.bodyEditor.getSession().getValue();
                self.view.find('.body').html(self.body);
                console.log(self.body);
            })
            
            this.render();
            return this;
        },

        render:function(){
            //Rendering stencil body using Transparency causes HTML to be escaped
            //so ask it to ignore body and add it in 'manually'
            this.view.render(this,{
                body:{
                    text: function(){
                        return "";
                    }
                }
            });
            this.bodyEditor.getSession().setValue(this.body || "");
            return this;
        },
        /*
         */
        theme_set:function(theme){
            if(this.theme) self.view.removeClass(this.theme.handle());
            this.theme = theme;
            this.view.addClass(this.theme.handle());
            return this;
        },
    });
    
    var Theme = Component.extend({
        constructor: function(id){
            this.base('theme',id);
        },
        
        get: function(){
            this.base(function(data){
                this.style = data.style;
            });
            return this;
        },
        
        put: function(){
            this.base({
                style : this.style
            });
            return this;
        },
        
        // Load the theme into the page by nesting its LESS within a rule 
        // that has its id as a class selector, parsing it, and then 
        // inserting the resulting CSS into the head
        load: function() {
            var self = this;
            if(!self.loaded){
                var parser = new less.Parser;
                parser.parse("."+self.handle()+" {\n"+self.style+"\n}", function(error, tree) {
                    if(error) {
                        return console.error(error)
                    }
                    $('head').append('<style id="'+self.handle()+'">\n'+tree.toCSS()+'\n</style>')
                    self.loaded = true;
                });
            }
            return self;
        },
        
        // Unload the theme from the page
        unload: function() {
            if(this.loaded){
                $('head').remove('style#'+this.handle());
                this.loaded = false;
            }
            return this;
        },
        
        show: function(){
            var styleHandle = this.handle()+"-style";

            this.view = $('<div class="theme">\
                <i class="icon-save"></i>\
                <div class="style" id="'+styleHandle+'"></div>\
            </div>').appendTo('body');
            
            this.styleEditor = Editor(styleHandle,'css');            
            
            var self = this;
            this.view.on('click','.icon-save',function(event){
                self.style = this.styleEditor.getSession().getValue();
                self.put();
                self.load();
            });
            
            this.render();
            return this;
        },

        render:function(){
            this.view.render(this);
            this.styleEditor.getSession().setValue(this.style);
            return this;
        },

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
