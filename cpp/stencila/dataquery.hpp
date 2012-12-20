/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file dataquery.hpp
//! @brief Definition of class Dataquery
//! @author Nokome Bentley

#pragma once

#include <string>
#include <vector>

#include <boost/format.hpp>
#include <boost/foreach.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string/join.hpp>

#include <stencila/exception.hpp>
#include <stencila/dataset.hpp>
#include <stencila/datatable.hpp>

namespace Stencila {

//! @class Element
//! @brief An element of a Dataquery
class Element {
public:

    //! @brief Get the column name for this Element
    //!
    //! Expressions can be used to create columns in the resultant Datatable.
    //! Those columns need a name, and this method provides that name.
    virtual std::string name(void) const {
        return "";
    }

    //! @brief Get the Data Query Language (DQL) representation of this Dataquery element
    //! @return The DQL for this query element
    virtual std::string dql(void) const {
        return "";
    }

    //! @brief Get the Structured Query Language (SQL) for this Dataquery element
    //! @param which Which version of SQL to produce
    //! @return The SQL for this query element
    virtual std::string sql(unsigned short which=0) const {
        return "";
    }
};

//! @class Expression
//! @brief An expression used in a Dataquery
class Expression : public Element {
public:

};

//! @class Column
//! @brief A Dataquery element which represents the column of a Datatable
class Column : public Expression {
private:

    //! @brief Name of the column
    std::string name_;

public:

    //! @brief Construct a column element
    //! @param name Name of the column
    Column(const std::string& name):
        name_(name){
    }

    virtual std::string name(void) const {
        return name_;
    }

    virtual std::string dql(void) const {
        return name_;
    }

    virtual std::string sql(unsigned short which=0) const {
        return '"' + name_ + '"';
    }
};

//! @class Constant
//! @brief A constant used in a Dataquery expression
template<typename Type> class Constant;

//! @class Constant<void>
//! @brief Base class for other class specialisations of templated class Constant
template<>
class Constant<void> : public Expression {

};

//! @class template<typename Type> Constant<Type>
//! @brief Specialisation for builtin types e.g. int, float
template<typename Type>
class Constant : public Constant<void> {
private:

    //! @brief Value of the constant
    Type value_;

public:

    //! @brief Construct a Constant
    //! @param value Value of the constant
    Constant(const Type& value):
        value_(value){
    }

    virtual std::string name(void) const {
        return boost::lexical_cast<std::string>(value_);
    }

    virtual std::string dql(void) const {
        return boost::lexical_cast<std::string>(value_);
    }

    virtual std::string sql(unsigned short which=0) const {
        return boost::lexical_cast<std::string>(value_);
    }
};

template<>
class Constant<std::string> : public Constant<void> {
private:

    //! @brief
    std::string value_;

public:

    //! @brief
    //! @param
    //! @return
    Constant(const std::string& value):
        value_(value){
    }

    virtual std::string name(void) const {
        return value_;
    }

    virtual std::string dql(void) const {
        return "'"+value_+"'";
    }

    virtual std::string sql(unsigned short which=0) const {
        return "'"+value_+"'";
    }
};

class Call : public Expression {
private:

    //! @brief
    std::string name_;

    //! @brief
    std::vector<Expression*> args_;

public:

    //! @brief
    Call(const std::string& name):
        name_(name){
    }

    //! @brief
    //! @param name
    //! @param exprs
    //! @return
    template<
        typename... Expressions
    >
    Call(const std::string& name,const Expressions&... exprs):
        name_(name){
        append_all(exprs...);
    }

    //! @brief
    //! @param expr
    //! @return
    Call& append(Expression* expr){
        args_.push_back(expr);
        return *this;
    }

    //! @brief
    //! @return
    Call& append_all(void){
        return *this;
    }

    //! @brief
    //! @param expr
    //! @param exprs
    //! @return
    template<
        typename Expression,
        typename... Expressions
    >
    Call& append_all(const Expression& expr,const Expressions&... exprs){
        append(new Expression(expr));
        return append_all(exprs...);
    }

    virtual std::string name(void) const {
        std::vector<std::string> args;
        BOOST_FOREACH(const Expression* arg, args_){
            args.push_back(arg->name());
        }
        return name_+"("+boost::algorithm::join(args, ", ")+")";
    }

    virtual std::string dql(void) const {
        std::vector<std::string> args;
        BOOST_FOREACH(const Expression* arg, args_){
            args.push_back(arg->dql());
        }
        return name_+"("+boost::algorithm::join(args, ", ")+")";
    }

    virtual std::string sql(unsigned short which=0) const {
        std::vector<std::string> args;
        BOOST_FOREACH(Element* arg, args_){
            args.push_back(arg->sql(which));
        }
        return name_+"("+boost::algorithm::join(args, ", ")+")";
    }
};

class Aggregate : public Expression {

private:

    //! @brief
    std::string name_;
    
    //! @brief
    Expression* expr_;

public:

    //! @brief
    //! @param name
    //! @param expr
    Aggregate(const std::string& name, Expression* expr):
        name_(name),
        expr_(expr){
    }

    //! @brief
    //! @param name
    //! @param expr
    //! @return
    template<class Expression>
    Aggregate(const std::string& name, const Expression& expr):
        name_(name),
        expr_(new Expression(expr)){
    }

    virtual std::string name(void) const {
        return name_+"("+expr_->name()+")";
    }

    std::string name(unsigned short which) const {
        if(which==1) return name_+"1_";
        else if(which==2) return name_+"2_";
    }

    virtual std::string dql(void) const {
        return name_+"("+expr_->dql()+")";
    }

    virtual std::string sql(unsigned short which=0) const {
        if(which==0) return name_ + "(" + expr_->sql(which) + ")";
        else if(which==1) return name_ + "1(" + expr_->sql(which) + ") AS "+name_+"1_";
        else if(which==2) return name_ + "2("+name_+"1_)";
        return "";
    }
};

class Operator : public Expression {
};

template<int Code>
class UnaryOperator : public Operator {

protected:

    //! @brief
    Expression* expr_;

public:

    //! @brief
    UnaryOperator(void):
        expr_(0){
    }

    //! @brief
    UnaryOperator(Expression* expr):
        expr_(expr){
    }

    //! @brief
    //! @param expr
    //! @return
    template<class Expression>
    UnaryOperator(const Expression& expr):
        expr_(new Expression(expr)){
    }

    virtual std::string name(void) const {
        return dql_symbol() + expr_->name();
    }

    virtual std::string dql(void) const {
        return dql_symbol() + expr_->dql();
    }

    virtual std::string sql(unsigned short which=0) const {
        return sql_symbol() + expr_->sql(which);
    }
    
    const char* dql_symbol(void) const;
    const char* sql_symbol(void) const;
};

#define UNOP(code,name,dql,sql) \
    typedef UnaryOperator<code> name; \
    template<> inline const char* name::dql_symbol(void) const {return dql;} \
    template<> inline const char* name::sql_symbol(void) const {return sql;}

UNOP(5,Positive,"+","+")
UNOP(6,Negative,"-","-")
UNOP(7,Not,"!","not")

#undef UNOP


template<int Code>
class BinaryOperator : public Operator {
protected:

    //! @brief
    Expression* left_;
    
    //! @brief
    Expression* right_;

public:

    //! @brief
    BinaryOperator(void):
        left_(0),
        right_(0){
    }

    //! @brief
    //! @param left
    //! @param right
    //! @return
    BinaryOperator(Expression* left, Expression* right):
        left_(left),
        right_(right){
    }

    //! @brief
    //! @param left
    //! @param right
    //! @return
    template<class Left, class Right>
    BinaryOperator(const Left& left, const Right& right):
        left_(new Left(left)),
        right_(new Right(right)){
    }

    virtual std::string name(void) const {
        return left_->name() + dql_symbol() + right_->name();
    }

    virtual std::string dql(void) const {
        std::string dql;
        
        std::string left = left_->dql();
        if(dynamic_cast<Operator* const>(left_)) dql += "(" + left + ")";
        else dql += left;
        
        dql += dql_symbol();
        
        std::string right = right_->dql();
        if(dynamic_cast<Operator* const>(right_)) dql += "(" + right + ")";
        else dql += right;
        
        return dql;
    }

    virtual std::string sql(unsigned short which=0) const {
        std::string sql;
        
        std::string left = left_->sql(which);
        if(dynamic_cast<Operator* const>(left_)) sql += "(" + left + ")";
        else sql += left;
        
        sql += sql_symbol();
        
        std::string right = right_->sql(which);
        if(dynamic_cast<Operator* const>(right_)) sql += "(" + right + ")";
        else sql += right;
        
        return sql;
    }
    
    //! @brief
    //! @return
    const char* dql_symbol(void) const;
    
    //! @brief
    //! @return
    const char* sql_symbol(void) const;
};

#define BINOP(code,name,dql,sql) \
    typedef BinaryOperator<code> name; \
    template<> inline const char* name::dql_symbol(void) const {return dql;} \
    template<> inline const char* name::sql_symbol(void) const {return sql;}

BINOP(10,Multiply,"*","*")
BINOP(11,Divide,"/","/")
BINOP(12,Add,"+","+")
BINOP(13,Subtract,"-","-")

BINOP(18,Equal,"==","==")
BINOP(19,NotEqual,"!=","!=")
BINOP(20,LessThan,"<","<")
BINOP(21,LessEqual,"<=","<=")
BINOP(22,GreaterThan,">",">")
BINOP(23,GreaterEqual,">=",">=")

BINOP(30,And," and "," AND ")
BINOP(31,Or," or "," OR ")

#undef BINOP

class As : public Element {

private:

    //! @brief
    Element* ele_;

    //! @brief
    std::string name_;

public:

    //! @brief
    //! @param ele
    //! @param name
    //! @return
    As(Element* ele,const std::string& name):
        ele_(ele),
        name_(name){
    }

    //! @brief
    //! @param name
    //! @return
    template<class Element>
    As(const Element& ele,const std::string& name):
        ele_(new Element(ele)),
        name_(name){
    }

    virtual std::string name(void) const {
        return name_;
    }

    virtual std::string dql(void) const {
        return "as(" + ele_->dql() + ",\"" + name_ + "\")";
    }

    virtual std::string sql(unsigned short which=0) const {
        return ele_->sql(which) + " AS \"" + name_ + "\"";
    }
};

class Distinct : public Element {
public:
};

class All : public Element {
public:
};

class Where : public Element {

private:
    Expression* expr_;

public:

    //! @brief
    //! @param expr
    Where(Expression* expr):
        expr_(expr){
    }

    //! @brief
    //! @param expr
    //! @return
    template<class Expression>
    Where(const Expression& expr):
        expr_(new Expression(expr)){
    }

    Expression& expression(void) const {
        return *expr_;
    }

    virtual std::string dql(void) const {
        return "where("+expr_->dql()+")";
    }

    virtual std::string sql(unsigned short which=0) const {
        return expr_->sql(which);
    }
};

class By : public Element {

private:

    //! @brief
    Element* ele_;

public:

    //! @brief
    //! @param ele
    By(Element* ele):
        ele_(ele){
    }

    //! @brief
    //! @param ele
    //! @return
    template<class Element>
    By(const Element& ele):
        ele_(new Element(ele)){
    }

    //! @brief
    //! @return
    Element* element(void) const {
        return ele_;
    }

    virtual std::string name(void) const {
        return ele_->name();
    }

    virtual std::string dql(void) const {
        return "by("+ele_->dql()+")";
    }

    virtual std::string sql(unsigned short which=0) const {
        return ele_->sql(which);
    }
};

class Having : public Element {

private:

    //! @brief
    Expression* expr_;

public:

    //! @brief
    //! @param expr
    Having(Expression* expr):
        expr_(expr){
    }

    //! @brief
    //! @param expr
    //! @return
    template<class Expression>
    Having(const Expression& expr):
        expr_(new Expression(expr)){
    }

    virtual std::string dql(void) const {
        return "having("+expr_->dql()+")";
    }

    virtual std::string sql(unsigned short which=0) const {
        return expr_->sql(which);
    }
};

class Order : public Element {

private:

    //! @brief
    Expression* expr_;
    
    //! @brief
    float dir_;

public:

    //! @brief
    //! @param expr
    //! @param dir
    Order(Expression* expr,const float& dir=1):
        expr_(expr),
        dir_(dir){
    }

    //! @brief
    //! @param dir
    //! @return
    template<class Expression>
    Order(const Expression& expr,const float& dir=1):
        expr_(new Expression(expr)),
        dir_(dir){
    }

    //! @brief
    //! @return
    float direction(void) const {
        return dir_;
    }

    virtual std::string dql(void) const {
        std::string dql = "order(" + expr_->dql();
        if(dir_!=1) dql += "," + boost::lexical_cast<std::string>(dir_);
        return dql + ")";
    }

    virtual std::string sql(unsigned short which=0) const {
        return expr_->sql(which);
    }
};

class Limit : public Element {
private:

    //! @brief
    Expression* expr_;
    
public:

    //! @brief
    //! @param expr
    Limit(Expression* expr):
        expr_(expr){
    }

    //! @brief
    //! @param expr
    //! @return
    template<class Expression>
    Limit(const Expression& expr):
        expr_(new Expression(expr)){
    }

    virtual std::string dql(void) const {
        return "limit("+expr_->dql()+")";
    }

    virtual std::string sql(unsigned short which=0) const {
        return expr_->sql(which);
    }
};

class Offset : public Element {
private:

    //! @brief
    Expression* expr_;

public:
    Offset(Expression* expr):
        expr_(expr){
    }

    //! @brief
    //! @param expr
    //! @return
    template<class Expression>
    Offset(const Expression& expr):
        expr_(new Expression(expr)){
    }

    virtual std::string dql(void) const {
        return "offset("+expr_->dql()+")";
    }

    virtual std::string sql(unsigned short which=0) const {
        return expr_->sql(which);
    }
};

class Combiner : public Element {
protected:
    
    //! @brief Element that is the subject of this combiner.
    //! @note Usually this will be a By element but if it is not then will be replaced by one in Dataquery::compile()
    Element* subject_;

public:

    //! @brief
    Combiner(Element* subject):
        subject_(subject){
    }

    //! @brief
    //! @return
    Element* subject(void) const {
        return subject_;
    }
    
    //! @brief
    //! @param subject
    //! @return
    Combiner& subject(Element* subject) {
        subject_ = subject;
        return *this;
    }

    //! @brief
    //! @param datatable
    virtual void combine(Datatable& datatable) const = 0;
};

class Top : public Combiner {
protected:

    //! @brief
    Aggregate* aggregate_;
    
    //! @brief
    unsigned int number_;

public:

    //! @brief
    //! @param subject
    //! @param aggregate
    //! @param num
    Top(Element* subject,Aggregate* aggregate,const unsigned int& num=10):
        Combiner(subject),
        aggregate_(aggregate),
        number_(num){
    }

    //! @brief
    //! @param subject
    //! @param aggregate
    //! @param num
    //! @return
    template<class Element, class Aggregate>
    Top(const Element& subject,const Aggregate& aggregate,const unsigned int& num=10):
        Combiner(new Element(subject)),
        aggregate_(new Aggregate(aggregate)),
        number_(num){
    }

    //! @brief
    //! @return
    Expression* aggregate(void) const {
        return aggregate_;
    }

    virtual void combine(Datatable& datatable) const {
        //Determine the top levels
        std::stringstream sql;
        std::string subject = '"'+subject_->name()+'"';
        std::string aggregate = '"'+aggregate_->name(2)+'"';
        std::string table = '"'+datatable.name()+'"';
        sql <<"UPDATE "<<table<<" SET "<<subject<<" = '<other>' WHERE "<<subject<<" IN ("
                <<"SELECT "<<subject<<" FROM "<<table<<" ORDER BY "<<aggregate_->sql(2)<<" DESC LIMIT "<<number_
            <<")";
        std::string sql_str = sql.str();
        datatable.execute(sql_str);
    }
    
};

class Margin : public Element {
protected:

    //! @brief  
    Element* subject_;

public:

    //! @brief 
    //! @param subject
    Margin(Element* subject):
        subject_(subject){
    }

    //! @brief 
    //! @return 
    Element* subject(void) const {
        return subject_;
    }
};

class Adjuster : public Element {
protected:

    //! @brief 
    std::vector<By*> bys_;

public:

    //! @brief 
    virtual void adjust(Datatable& table) const = 0;
};

class Proportion : public Adjuster {
public:

    //! @brief 
    //! @param table
    void adjust(Datatable& table) const {
        //! @todo
        //Calculate sums for each by
        //Call* sum = Aggregate("sum");
        //Columns cols = {sum};
        //sql(table,,bys_);
    };
};

class Reshaper : public Element {

};

//! @class Dataquery
//! @todo Document fully
class Dataquery : public Element {
private:

    //! @brief 
    std::vector<Element*> elements_;
    
    //! @brief 
    std::string from_;

    //! @brief 
    bool compiled_;

    typedef std::vector<std::pair<std::string,const Element*>> Columns;

    //! @brief 
    bool distinct_;
    
    //! @brief 
    std::vector<const Element*> values_;
    
    //! @brief 
    std::vector<const Where*> wheres_;
    
    //! @brief 
    std::vector<const By*> bys_;
    
    //! @brief 
    std::vector<const Having*> havings_;
    
    //! @brief 
    std::vector<const Order*> orders_;
    
    //! @brief 
    const Limit* limit_;
    
    //! @brief 
    const Offset* offset_;

    //! @brief 
    std::vector<const Combiner*> combiners_;
    
    //! @brief 
    std::vector<const Margin*> margins_;
    
    //! @brief 
    std::vector<const Adjuster*> adjusters_;
    
    //! @brief 
    std::vector<const Reshaper*> reshapers_;

public:

    //! @brief
    Dataquery(void):
        from_("<from>"){
    }

    //! @brief
    //! @param elements
    //! @return
    template<class... Elements>
    Dataquery(const Elements&... elements):
        from_("<from>"){
        append_all(elements...);
    }

    //! @name Append elements
    //! @brief Append elements to the dataquery
    //! @{
    
    //! @brief 
    //! @param ele
    Dataquery& append(Element* ele){
        elements_.push_back(ele);
        compiled_ = false;
        return *this;
    }

    //! @brief
    //! @return
    Dataquery& append_all(void){
        return *this;
    }

    //! @brief
    //! @param eles
    //! @return
    template<
        typename Element,
        typename... Elements
    >
    Dataquery& append_all(const Element& ele,const Elements&... eles){
        append(new Element(ele));
        return append_all(eles...);
    }

    //! @}

    //! @brief 
    //! @param from
    Dataquery& from(const std::string& from){
        from_ = from;
        return *this;
    }

    //! @brief
    //! @return
    Dataquery& compile(void){
        if(not compiled_){
            //Reset members
            distinct_ = false;
            values_.clear();
            wheres_.clear();
            bys_.clear();
            havings_.clear();
            orders_.clear();
            limit_ = 0;
            offset_ = 0;
            
            BOOST_FOREACH(Element* element, elements_){
                if(dynamic_cast<const Distinct*>(element)){
                    distinct_ = true;
                }
                else if(dynamic_cast<const All*>(element)){
                    distinct_ = false;
                }
                else if(const Where* where = dynamic_cast<const Where*>(element)){
                    wheres_.push_back(where);
                }
                else if(const By* by = dynamic_cast<const By*>(element)){
                    bys_.push_back(by);
                }
                else if(const Having* having = dynamic_cast<const Having*>(element)){
                    havings_.push_back(having);
                }
                else if(const Order* order = dynamic_cast<const Order*>(element)){
                    orders_.push_back(order);
                }
                else if(const Limit* limit = dynamic_cast<const Limit*>(element)){
                    limit_ = limit;
                }
                else if(const Offset* offset = dynamic_cast<const Offset*>(element)){
                    offset_ = offset;
                }
                else if(Combiner* combiner = dynamic_cast<Combiner*>(element)){
                    Element* subject = combiner->subject();
                    By* by = dynamic_cast<By*>(subject);
                    if(!by) combiner->subject(new By(subject));
                    
                    combiners_.push_back(combiner);
                }
                else {
                    values_.push_back(element);
                }
            }
            
            compiled_ = true;
        }
        return *this;
    }

    std::string dql(void) {
        compile();
        std::string dql;
        for(auto i=elements_.begin();i!=elements_.end();i++){
            dql += (*i)->dql();
            if(i!=elements_.end()-1) dql += ",";
        }
        return dql;
    }

    //! @brief
    //! @param table
    //! @param distinct
    //! @param columns
    //! @param where
    //! @param by
    //! @param having
    //! @param order
    //! @param limit_offset
    //! @return
    static std::string sql(const Datatable& table, 
        const std::string& distinct, const std::string& columns,
        const std::string& where = "", const std::string& by = "",
        const std::string& having = "", const std::string& order = "",
        const std::string& limit_offset = ""
    ){
        return "SELECT " +  distinct + columns + 
                " FROM \"" + table.name() + "\"" +
                where + by + having + order + limit_offset;
    }

    //! @brief
    //! @param distinct
    //! @return
    static std::string sql_distinct(bool distinct){
        if(distinct) return " DISTINCT";
        else return "";
    }

    //! @brief
    //! @param columns
    //! @param which
    //! @return
    static std::string sql_columns(const Columns columns, unsigned short which = 0){
        std::string sql = "";
        if(columns.size()==0){
            sql += " *";
        } else {
            sql += " ";
            for(auto i=columns.begin();i!=columns.end();i++){
                sql += i->second->sql(which) + " AS " + i->first;
                if(i!=columns.end()-1) sql += ", ";
            }
        }
        return sql;
    }

    //! @brief
    //! @param wheres
    //! @return
    static std::string sql_where(const std::vector<const Where*>& wheres){
        std::string sql = "";
        if(wheres.size()>0){
            sql += " WHERE ";
            if(wheres.size()>1) sql += "(";
            for(auto i=wheres.begin();i!=wheres.end();i++){
                sql += (*i)->sql();
                if(i!=wheres.end()-1) sql += ") AND (";
            }
            if(wheres.size()>1) sql += ")";
        }
        return sql;
    }

    //! @brief 
    //! @param bys
    //! @return 
    static std::string sql_by(const std::vector<const By*>& bys){
        std::string sql = "";
        if(bys.size()>0){
            sql += " GROUP BY ";
            for(auto i=bys.begin();i!=bys.end();i++){
                sql += (*i)->sql();
                if(i!=bys.end()-1) sql += ", ";
            }
        }
        return sql;
    }

    //! @brief 
    //! @param havings
    //! @return 
    static std::string sql_having(const std::vector<const Having*>& havings){
        std::string sql = "";
        if(havings.size()>0){
            sql += " HAVING ";
            if(havings.size()>1) sql += "(";
            for(auto i=havings.begin();i!=havings.end();i++){
                sql += (*i)->sql();
                if(i!=havings.end()-1) sql += ") AND (";
            }
            if(havings.size()>1) sql += ")";
        }
        return sql;
    }

    //! @brief 
    //! @param orders
    //! @return 
    static std::string sql_order(const std::vector<const Order*>& orders){
        std::string sql = "";
        if(orders.size()>0){
            sql += " ORDER BY ";
            for(auto i=orders.begin();i!=orders.end();i++){
                const Order* order = *i;
                sql += order->sql();
                if(order->direction()>0) sql += " ASC";
                else if(order->direction()<0) sql += " DESC";
                if(i!=orders.end()-1) sql += ", ";
            }
        }
        return sql;
    }

    //! @brief 
    //! @param limit
    //! @param offset
    //! @return 
    static std::string sql_limit_offset(const Limit* limit, const Offset* offset){
        std::string sql = "";
        if(limit){
            sql += " LIMIT " + limit->sql();
        }
        if(offset){
            //Offset can only come after a limit clause. So add one if not present.
            //The theoretical maximum number of rows in an SQLite database
            //is 2^64 = 18446744073709551616 (see http://www.sqlite.org/limits.html)
            //However SQLite baulks at such a large integer in an limit clause so instead
            //we have to use the maximum value for an integer: 2^64/2
            if(not limit) sql += " LIMIT 9223372036854775807";
            sql += " OFFSET " + offset->sql();
        }
        return sql;
    }

    //! @brief Execute this Dataquery on a Datatable
    //! @param table The table to execute this query on
    //! @return A datatable produced by this query
    Datatable execute(Datatable& table){
        Columns columns;
        
        if(combiners_.size()==0 and margins_.size()==0 and adjusters_.size()==0){
            // Select data
            // Since their are no modifiers do a single pass
            return table.select(sql(table,
                sql_distinct(distinct_), sql_columns(columns,0), 
                sql_where(wheres_), sql_by(bys_),
                sql_having(havings_), sql_order(orders_),
                sql_limit_offset(limit_,offset_)
            ));
        } else {
            // Select data
            // As as a first pass, obtain the necessary columns applying any wheres
            // Note that which==1
            std::string first_sql = sql(table,
                "",sql_columns(columns,1),
                sql_where(wheres_)
            );
            // Execute first_sql with cache reuse, but no caching
            Datatable first = table.select(first_sql,true,false);
            
            // Apply combiners
            // Combiners set values in corresponding columns to <other>
            for(const Combiner* combiner : combiners_){
                combiner->combine(first);
            }
            
            // Value calculations
            // Calculate values using which==2
            std::string second_sql = sql(first,
                "",sql_columns(columns,2)
            );
            Datatable second = first.select(second_sql,true,false);
                
            // Margin calculations
            // Each margin needs to be calculated by dropping "its" By from bys_
            for(const Margin* margin : margins_){
                std::vector<const By*> bys;
                Columns columns;
                for(const By* by : bys_){
                    std::string name = margin->subject()->name();
                    if(by->name()==name) {
                        Constant<std::string>* label = new Constant<std::string>("<all>");
                        columns.push_back(std::make_pair(name,label));
                    }
                    else {
                        bys.push_back(by);
                        columns.push_back(std::make_pair(name,by->element()));
                    }
                }
                for(const Element* column : values_) {
                    columns.push_back(std::make_pair(column->name(),column));
                }
                std::string alls_sql = sql(second,
                    "",sql_columns(columns,2),
                    "",sql_by(bys)
                );
                // Execute the SQL and insert the resultant table
                Datatable alls = first.select(alls_sql);
                // Append the alls to the values
                second.append(alls);
            }
            
            // Apply adjusters
            for(const Adjuster* adjuster : adjusters_){
                // Each adjuster needs to calculate an overall value(s)
                // and then do adjustment
                adjuster->adjust(second);
            }
        }
    }
};

}
