#pragma once

#include <stack>
#include <string>

#include <stencila/exception.hpp>
#include <stencila/contexts/context.hpp>

namespace Stencila {
namespace Contexts {

class Map : public Context<Map> {
private:

    class Frame {
    private:
        std::string value_;
        std::map<std::string,Frame*> children_;

    public:
        Frame(const std::string& value=""):
            value_(value){
        }

        std::string value(void) const {
            return value_;
        }

        std::map<std::string,Frame*>::iterator begin(void){
            return children_.begin();
        }

        std::map<std::string,Frame*>::iterator end(void){
            return children_.end();
        }

        Frame* set(const std::string& name,const std::string& value){
            Frame* frame = new Frame(value);
            children_[name] = frame;
            return frame;
        }

        Frame* get(const std::string& name) const {
            auto i = children_.find(name);
            if(i!=children_.end()) return i->second;
            else return 0;
        }
    };

    Frame frame_;
    std::list<Frame*> frames_;

    void set_(const std::string& name, const std::string& value){
        frames_.back()->set(name,value);
    }

    Frame* get_(const std::string& name){
        for(auto frame : frames_){
            Frame* got = frame->get(name);
            if(got) return got;
        }
        throw Exception("Name not found: "+name);
    }

    std::stack<Frame*> subjects_;

    struct Loop {
        Frame frame;
        std::string name;
        std::map<std::string,Frame*>::iterator iterator;
        std::map<std::string,Frame*>::iterator end;

        Loop(const std::string& item,Frame* items){
            name = item;
            iterator = items->begin();
            end = items->end();
        }
    };
    std::stack<Loop*> loops_;

public:

    Map(void){
        frames_.push_back(&frame_);
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
        Frame* items = get_(expression);

        Loop* loop = new Loop(item,items);
        loops_.push(loop);

        frames_.push_back(&loop->frame);

        if(loop->iterator!=loop->end){
            loop->frame.set(loop->name,loop->iterator->second->value());
            return true;
        }
        else return false;
    }

    bool next(void){
        Loop* loop = loops_.top();
        loop->iterator++;
        if(loop->iterator==loop->end) return false;
        else {
            loop->frame.set(loop->name,loop->iterator->second->value());
            return true;
        }
    }

    Map& end(void){
        delete loops_.top();
        loops_.pop();
        
        frames_.pop_back();
        return *this;
    }

   
    Map& enter(const std::string& expression=""){
        frames_.push_back(get_(expression));
        return *this;
    }

    Map& exit(void){
        frames_.pop_back();
        return *this;
    }

};

}
}
