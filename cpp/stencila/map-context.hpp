#pragma once

#include <stack>
#include <string>

#include <stencila/exception.hpp>
#include <stencila/context.hpp>

namespace Stencila {

/**
 * @todo Document
 * @todo Leaks memory, need to balance up all the news or else not use new so much.
 */
class MapContext : public Context {
private:

    class Namespace {
    private:
        std::string value_;
        std::map<std::string,Namespace*> children_;

    public:
        Namespace(const std::string& value=""):
            value_(value){
        }

        std::string value(void) const {
            return value_;
        }

        std::map<std::string,Namespace*>::iterator begin(void){
            return children_.begin();
        }

        std::map<std::string,Namespace*>::iterator end(void){
            return children_.end();
        }

        Namespace* set(const std::string& name,const std::string& value){
            Namespace* ns = new Namespace(value);
            children_[name] = ns;
            return ns;
        }

        Namespace* get(const std::string& name) const {
            auto i = children_.find(name);
            if(i!=children_.end()) return i->second;
            else return 0;
        }
    };

    Namespace ns_;
    std::list<Namespace*> nss_;

    void set_(const std::string& name, const std::string& value){
        nss_.back()->set(name,value);
    }

    Namespace* get_(const std::string& name) const {
        for(auto ns : nss_){
            Namespace* got = ns->get(name);
            if(got) return got;
        }
        if(parent_) return parent_->get_(name);
        else throw Exception("Name not found: "+name);
    }

    std::stack<Namespace*> subjects_;

    struct Loop {
        Namespace ns;
        std::string name;
        std::map<std::string,Namespace*>::iterator iterator;
        std::map<std::string,Namespace*>::iterator end;

        Loop(const std::string& item,Namespace* items){
            name = item;
            iterator = items->begin();
            end = items->end();
        }
    };
    std::stack<Loop*> loops_;

    const MapContext* parent_ = 0;

public:

    MapContextContext(void){
        nss_.push_back(&ns_);
    }

    bool accept(const std::string& language) const {
        // Return false since this context does not
        // accept any code
        return false;
    }

    void execute(const std::string& code){
        unsupported_("execute");
    }
    
    std::string interact(const std::string& code){
        unsupported_("interact");
    }

    void assign(const std::string& name, const std::string& expression){
        set_(name,expression);
    }

    std::string write(const std::string& expression){
        return get_(expression)->value();
    }

    std::string paint(const std::string& format,const std::string& code){
        unsupported_("paint");
    }

    bool test(const std::string& expression){
        std::string value = write(expression);
        return value.length()>0;
    }

    void mark(const std::string& expression){
        subjects_.push(get_(expression));
    }

    bool match(const std::string& expression){
        if(subjects_.size()>0){
            return subjects_.top()->value()==expression;
        } 
        else throw Exception("No subject has been set");
    }

    void unmark(void){
        subjects_.pop();
    }

    bool begin(const std::string& item,const std::string& expression){
        Namespace* items = get_(expression);

        Loop* loop = new Loop(item,items);
        loops_.push(loop);

        nss_.push_back(&loop->ns);

        if(loop->iterator!=loop->end){
            loop->ns.set(loop->name,loop->iterator->second->value());
            return true;
        }
        else return false;
    }

    bool next(void){
        Loop* loop = loops_.top();
        loop->iterator++;
        if(loop->iterator==loop->end) return false;
        else {
            loop->ns.set(loop->name,loop->iterator->second->value());
            return true;
        }
    }

    void end(void){
        delete loops_.top();
        loops_.pop();
        
        nss_.pop_back();
    }

   
    void enter(void){
        nss_.push_back(new Namespace);
    }

    void enter(const std::string& expression){
        nss_.push_back(get_(expression));
    }

    void exit(void){
        nss_.pop_back();
    }

};

}
