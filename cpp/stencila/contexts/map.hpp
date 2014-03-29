#pragma once

#include <stack>
#include <string>

#include <stencila/exception.hpp>
#include <stencila/contexts/context.hpp>

namespace Stencila {
namespace Contexts {

/**
 * @todo Document
 * @todo Leaks memory, need to balance up all the news or else not use new so much.
 */
class Map : public Context<Map> {
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

    const Map* parent_ = 0;

public:

    Map(void){
        nss_.push_back(&ns_);
    }

    std::string type(void) const {
        return "map-context";
    }

    Map& execute(const std::string& code){
        unsupported();
    }
    
    std::string interact(const std::string& code){
        unsupported();
    }

    Map& assign(const std::string& name, const std::string& expression){
        set_(name,expression);
        return *this;
    }

    std::string text(const std::string& expression){
        return get_(expression)->value();
    }

    std::string image(const std::string& format,const std::string& code){
        unsupported();
    }

    bool test(const std::string& expression){
        std::string value = text(expression);
        return value.length()>0;
    }

    Map& subject(const std::string& expression){
        subjects_.push(get_(expression));
        return *this;
    }

    bool match(const std::string& expression){
        if(subjects_.size()>0){
            return subjects_.top()->value()==expression;
        } 
        else throw Exception("No subject has been set");
    }

    Map& unsubject(void){
        subjects_.pop();
        return *this;
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

    Map& end(void){
        delete loops_.top();
        loops_.pop();
        
        nss_.pop_back();
        return *this;
    }

   
    Map& enter(void){
        nss_.push_back(new Namespace);
        return *this;
    }

    Map& enter(const std::string& expression){
        nss_.push_back(get_(expression));
        return *this;
    }

    Map& exit(void){
        nss_.pop_back();
        return *this;
    }

};

}
}
