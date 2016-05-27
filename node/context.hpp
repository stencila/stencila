#pragma once

#include <boost/lexical_cast.hpp>

#include <stencila/context.hpp>
#include <stencila/exception.hpp>

#include <nan.h>

#include "extension.hpp"

/**
 * A Node.js execution context
 *
 * Extends the abstract base class Context defined in `cpp/stencila/context.hpp`
 */
class NodeContext : public Stencila::Context {
public:
    
    NodeContext(const Nan::FunctionCallbackInfo<v8::Value>& info){
        context_.Reset(info[0].As<v8::Object>());
    }

    ~NodeContext(void) {
        context_.Reset();
    }

    /**
     * Implementation of pure virtual functions defined by `Context` class.
     * see there for documentation.
     */

    std::string details(void) const {
        return "NodeContext at " + string(this);
    };

    bool accept(const std::string& language) const {
        return language=="js";
    }

    std::string execute(
        const std::string& code, 
        const std::string& id="", 
        const std::string& format="", 
        const std::string& width="", 
        const std::string& height="", 
        const std::string& units=""
    ){
        return call_("execute",code,id,format,width,height,units);
    }

    std::string interact(const std::string& code, const std::string& id=""){
        return call_("interact",code);
    }

    void assign(const std::string& name, const std::string& expression){
        call_("assign",name,expression);
    }

    void input(const std::string& name, const std::string& type, const std::string& value){
        call_("input",name,type,value);
    };

    std::string write(const std::string& expression){
        return call_("write",expression);
    }

    bool test(const std::string& expression){
        return boost::lexical_cast<bool>(call_("test",expression));
    }

    void mark(const std::string& expression){
        call_("mark",expression);
    }

    bool match(const std::string& expression){
        return boost::lexical_cast<bool>(call_("match",expression));
    }

    void unmark(void){
        call_("unmark");
    }

    bool begin(const std::string& item,const std::string& items){
        return boost::lexical_cast<bool>(call_("begin",item,items));
    }

    bool next(void){
        return boost::lexical_cast<bool>(call_("next"));
    }

    void enter(const std::string& expression=""){
        call_("enter",expression);
    }

    void exit(void){
        call_("exit");
    }

private:
    /**
     * The object which represents this context on the Javascript "side"
     */
    Nan::Persistent<v8::Object> context_;

    /**
     * Call a method of the Javascript context object
     */
    template<typename... Args>
    std::string call_(const char* name, Args... args) {
        // Create a local handle to the persisted context
        v8::Local<v8::Object> context = Nan::New(context_);
        // Get the method
        v8::Local<v8::Function> method = v8::Handle<v8::Function>::Cast(
            context->Get(Nan::New(name).ToLocalChecked())
        );
        // Construct an array of arguments and call
        const int argc = sizeof...(args);
        v8::Local<v8::Value>* argv = new v8::Local<v8::Value>[argc];
        args_(argv, 0, args...);
        Nan::TryCatch catcher;
        v8::Local<v8::Value> result = method->Call(context, argc, argv);
        // Check for an exception
        if (catcher.HasCaught()) {
            //catcher.Message().As<String>();
            throw Stencila::Exception("Some exception");
        }
        // Convert to result to string
        std::string string;
        if (result->IsString()) {
            string = to<std::string>(result);
        }
        return string;
    }

    /**
     * Add arguments to array for call to Javascript method
     */
    template<typename Arg, typename... Args>
    void args_(v8::Local<v8::Value>* argv, int index, Arg arg, Args... args) {
        // Add first argumnet
        argv[index] = Nan::New(arg).ToLocalChecked();
        // Recurse to add the remaining args
        args_(argv, index+1, args...);
    }
    void args_(v8::Local<v8::Value>* argv, int index) {
        // End of recursion, nothing to do here.
    }

};
