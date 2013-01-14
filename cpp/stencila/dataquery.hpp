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
    
class Element;
typedef std::vector<const Element*> Elements;

//! @class Element
//! @brief An element of a Dataquery
class Element {
public:

    //! @brief Get the Data Query Language (DQL) representation of this Dataquery element
    //! @return The DQL for this query element
    virtual std::string dql(void) const {
        return "";
    }

    //! @brief Get the column name for this Element
    //!
    //! Elements can be used to create columns in the resultant Datatable.
    //! Those columns need a name, and this method provides that name.
    virtual std::string label(void) const {
        return "";
    };
    
    virtual std::string id(void) const {
        return boost::lexical_cast<std::string>(this);
    }

    //! @brief Get the Structured Query Language (SQL) for this Dataquery element
    //! @param which Which version of SQL to produce
    //! @return The SQL for this query element
    virtual std::string sql(unsigned short phase) const {
        return "";
    }
    
    //! @brief
    //! @param values
    //! @param which
    //! @return
    static std::string sql(const Elements& elements, unsigned short phase){
        std::string sql = "";
        if(elements.size()==0){
            sql += " *";
        } else {
            sql += " ";
            for(const Element* element : elements){
                sql += element->sql(phase) + " AS \"" + ((phase==0 or phase==10)?(element->label()):(element->id())) + "\"";
                if(element!=*(elements.end()-1)) sql += ", ";
            }
        }
        return sql;
    }
};

//! @class Column
//! @brief A Dataquery element which represents the column of a Datatable
class Column : public Element {
private:

    //! @brief Name of the column
    std::string name_;

public:

    //! @brief Construct a column element
    //! @param name Name of the column
    Column(const std::string& name):
        name_(name){
    }

    virtual std::string dql(void) const {
        return name_;
    }

    virtual std::string label(void) const {
        return name_;
    }

    virtual std::string sql(unsigned short phase) const {
        return '"' + name_ + '"';
    }
};

//! @class Constant
//! @brief A constant used in a Dataquery element
template<typename Type> class Constant;

//! @class Constant<void>
//! @brief Base class for other class specialisations of templated class Constant
template<>
class Constant<void> : public Element {

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

    virtual std::string dql(void) const {
        return boost::lexical_cast<std::string>(value_);
    }

    virtual std::string label(void) const {
        return boost::lexical_cast<std::string>(value_);
    }

    virtual std::string sql(unsigned short phase) const {
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

    virtual std::string dql(void) const {
        return "'"+value_+"'";
    }

    virtual std::string label(void) const {
        return value_;
    }

    virtual std::string sql(unsigned short phase) const {
        return "'"+value_+"'";
    }
};

class Call : public Element {
private:

    //! @brief
    std::string name_;

    //! @brief
    std::vector<Element*> args_;

public:

    //! @brief
    Call(const std::string& name):
        name_(name){
    }

    //! @brief
    //! @param name
    //! @param elements
    //! @return
    template<
        typename... Elements
    >
    Call(const std::string& name,const Elements&... elements):
        name_(name){
        append_all(elements...);
    }

    //! @brief
    //! @param expr
    //! @return
    Call& append(Element* element){
        args_.push_back(element);
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
        typename Element,
        typename... Elements
    >
    Call& append_all(const Element& element,const Elements&... elements){
        append(new Element(element));
        return append_all(elements...);
    }

    virtual std::string dql(void) const {
        std::vector<std::string> args;
        for(const Element* arg : args_){
            args.push_back(arg->dql());
        }
        return name_+"("+boost::algorithm::join(args, ", ")+")";
    }

    virtual std::string label(void) const {
        std::vector<std::string> args;
        for(const Element* arg : args_){
            args.push_back(arg->label());
        }
        return name_+"("+boost::algorithm::join(args, ", ")+")";
    }

    virtual std::string sql(unsigned short phase) const {
        std::vector<std::string> args;
        for(Element* arg : args_){
            args.push_back(arg->sql(phase));
        }
        return name_+"("+boost::algorithm::join(args, ", ")+")";
    }
};

class Aggregate : public Element {

private:

    //! @brief
    std::string name_;
    
    //! @brief
    Element* element_;

public:

    //! @brief
    //! @param name
    //! @param expr
    Aggregate(const std::string& name, Element* element):
        name_(name),
        element_(element){
    }

    virtual std::string dql(void) const {
        return name_+"("+element_->dql()+")";
    }

    virtual std::string label(void) const {
        return name_+"("+element_->label()+")";
    }

    virtual std::string sql(unsigned short phase) const {
        switch(phase){
            case 0: return name_ + "(" + element_->sql(phase) + ")"; break;
            case 1: return name_ + "1(" + element_->sql(phase) + ")"; break;
            case 2: return name_ + "2(\"" + id() + "\")"; break;
            case 3: return name_ + "(\"" + element_->id() + "\")"; break;
            case 10: return "\"" + id() + "\""; break;
        }
        return "";
    }
};

class Operator : public Element {
};

template<int Code>
class UnaryOperator : public Operator {

protected:

    //! @brief
    Element* element_;

public:

    //! @brief
    UnaryOperator(Element* element = 0):
        element_(element){
    }

    virtual std::string dql(void) const {
        return dql_symbol() + element_->dql();
    }

    virtual std::string label(void) const {
        return dql_symbol() + element_->label();
    }

    virtual std::string sql(unsigned short phase) const {
        return sql_symbol() + element_->sql(phase);
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
    Element* left_;
    
    //! @brief
    Element* right_;

public:

    //! @brief
    //! @param left
    //! @param right
    //! @return
    BinaryOperator(Element* left=0, Element* right=0):
        left_(left),
        right_(right){
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

    virtual std::string label(void) const {
        return left_->label() + dql_symbol() + right_->label();
    }

    virtual std::string sql(unsigned short phase) const {
        std::string sql;
        
        std::string left = left_->sql(phase);
        if(dynamic_cast<Operator* const>(left_)) sql += "(" + left + ")";
        else sql += left;
        
        sql += sql_symbol();
        
        std::string right = right_->sql(phase);
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
    Element* element_;

    //! @brief
    std::string name_;

public:

    //! @brief
    //! @param element
    //! @param name
    //! @return
    As(Element* element,const std::string& name):
        element_(element),
        name_(name){
    }

    virtual std::string dql(void) const {
        return "as(" + element_->dql() + ",\"" + name_ + "\")";
    }

    virtual std::string label(void) const {
        return name_;
    }

    virtual std::string sql(unsigned short phase) const {
        return element_->sql(phase);
    }
};

class Distinct : public Element {
public:

    virtual std::string dql(void) const {
        return "distinct()";
    }

    //! @brief Generate a SQL DISTINCT clause
    //! @param distinct Whether or not the select is DISTINCT
    //! @return SQL
    static std::string sql(bool distinct,unsigned short phase){
        if(distinct) return " DISTINCT";
        else return "";
    }
};

class All : public Element {
public:

    virtual std::string dql(void) const {
        return "all()";
    }
};


class Where;
typedef std::vector<const Where*> Wheres;

class Where : public Element {

private:
    Element* element_;

public:

    //! @brief
    //! @param expr
    Where(Element* element):
        element_(element){
    }

    virtual std::string dql(void) const {
        return "where("+element_->dql()+")";
    }

    virtual std::string sql(unsigned short phase) const {
        return element_->sql(phase);
    }
    
    //! @brief Generate a WHERE clause for a set of Wheres
    //! @param wheres
    //! @return
    static std::string sql(const Wheres& wheres, unsigned short phase){
        std::string sql = "";
        if(wheres.size()>0){
            sql += " WHERE ";
            if(wheres.size()>1) sql += "(";
            for(auto i=wheres.begin();i!=wheres.end();i++){
                sql += (*i)->sql(phase);
                if(i!=wheres.end()-1) sql += ") AND (";
            }
            if(wheres.size()>1) sql += ")";
        }
        return sql;
    }
};

class By;
typedef std::vector<const By*> Bys;

class By : public Element {

private:

    //! @brief
    const Element* element_;

public:

    //! @brief
    //! @param ele
    By(const Element* element):
        element_(element){
    }

    virtual std::string dql(void) const {
        return "by("+element_->dql()+")";
    }

    virtual std::string label(void) const {
        return element_->label();
    }

    virtual std::string sql(unsigned short phase) const {
        if(phase<2) return element_->sql(phase);
        else return '"'+id()+'"';
    }
    
    //! @brief Generate a GROUP BY clause for a set of Bys
    //! @param bys
    //! @return 
    static std::string sql(const Bys& bys, unsigned short phase){
        std::string sql = "";
        if(bys.size()>0){
            sql += " GROUP BY ";
            for(auto i=bys.begin();i!=bys.end();i++){
                sql += (*i)->sql(phase);
                if(i!=bys.end()-1) sql += ", ";
            }
        }
        return sql;
    }
};

class Having;
typedef std::vector<const Having*> Havings;

class Having : public Element {

private:

    //! @brief
    Element* element_;

public:

    //! @brief
    //! @param expr
    Having(Element* element):
        element_(element){
    }

    virtual std::string dql(void) const {
        return "having("+element_->dql()+")";
    }

    virtual std::string sql(unsigned short phase) const {
        return element_->sql(phase);
    }
    
    //! @brief 
    //! @param havings
    //! @return 
    static std::string sql(const Havings& havings,unsigned short phase){
        std::string sql = "";
        if(havings.size()>0){
            sql += " HAVING ";
            if(havings.size()>1) sql += "(";
            for(auto i=havings.begin();i!=havings.end();i++){
                sql += (*i)->sql(phase);
                if(i!=havings.end()-1) sql += ") AND (";
            }
            if(havings.size()>1) sql += ")";
        }
        return sql;
    }
};

class Order;
typedef std::vector<const Order*> Orders;

class Order : public Element {

private:

    //! @brief
    Element* element_;
    
    //! @brief
    float direction_;

public:

    //! @brief
    //! @param expr
    //! @param dir
    Order(Element* element,const float& direction=1):
        element_(element),
        direction_(direction){
    }
    
    float direction(void) const {
        return direction_;
    }

    virtual std::string dql(void) const {
        std::string dql = "order(" + element_->dql();
        if(direction_!=1) dql += "," + boost::lexical_cast<std::string>(direction_);
        return dql + ")";
    }

    virtual std::string sql(unsigned short phase) const {
        return element_->sql(phase);
    }
    

    //! @brief 
    //! @param orders
    //! @return 
    static std::string sql(const Orders& orders,unsigned short phase){
        std::string sql = "";
        if(orders.size()>0){
            sql += " ORDER BY ";
            for(auto i=orders.begin();i!=orders.end();i++){
                const Order* order = *i;
                sql += order->sql(phase);
                if(order->direction()>0) sql += " ASC";
                else if(order->direction()<0) sql += " DESC";
                if(i!=orders.end()-1) sql += ", ";
            }
        }
        return sql;
    }
};

class Limit;
class Offset;

class LimitOffset : public Element {
protected:

    //! @brief
    unsigned int number_;

public:

    LimitOffset(unsigned int number):
        number_(number){
    }

    unsigned int number(void) const {
        return number_;
    }
    
    std::string number_string(void) const {
        return boost::lexical_cast<std::string>(number_);
    }

    //! @brief 
    //! @param limit
    //! @param offset
    //! @return 
    static std::string sql(const Limit* limit, const Offset* offset);
};

class Limit : public LimitOffset {
public:

    Limit(unsigned int number):
        LimitOffset(number){
    }

    virtual std::string dql(void) const {
        return "limit("+number_string()+")";
    }
};

class Offset : public LimitOffset {
public:

    Offset(unsigned int number):
        LimitOffset(number){
    }
    
    virtual std::string dql(void) const {
        return "offset("+number_string()+")";
    }
};

std::string LimitOffset::sql(const Limit* limit, const Offset* offset){
    std::string sql = "";
    if(limit){
        sql += " LIMIT " + limit->number_string();
    }
    if(offset){
        //Offset can only come after a limit clause. So add one if not present.
        //The theoretical maximum number of rows in an SQLite database
        //is 2^64 = 18446744073709551616 (see http://www.sqlite.org/limits.html)
        //However SQLite baulks at such a large integer in an limit clause so instead
        //we have to use the maximum value for an integer: 2^64/2
        if(not limit) sql += " LIMIT 9223372036854775807";
        sql += " OFFSET " + offset->number_string();
    }
    return sql;
}


class Select {
private:

    bool distinct_;
    Elements columns_; 
    Wheres wheres_; 
    Bys bys_; 
    Havings havings_; 
    Orders orders_; 
    const Limit* limit_;
    const Offset* offset_;

public:

    Select& distinct(bool distinct){
        distinct_ = distinct;
        return *this;
    }
    
    Select& columns(const Elements& elements){
        columns_ = elements;
        return *this;
    }
    
    Select& where(const Wheres& wheres){
        wheres_ = wheres;
        return *this;
    }
    
    Select& by(const Bys& bys){
        bys_ = bys;
        return *this;
    }
    
    Select& having(const Havings& havings){
        havings_ = havings;
        return *this;
    }
    
    Select& order(const Orders& orders){
        orders_ = orders;
        return *this;
    }

    Select& limit(const Limit* limit){
        limit_ = limit;
        return *this;
    }
    
    Select& offset(const Offset* offset){
        offset_ = offset;
        return *this;
    }

    std::string sql(const Datatable& table, unsigned short int phase) const {
        return "SELECT " + 
                Distinct::sql(distinct_,phase) + 
                Element::sql(columns_,phase) + 
                " FROM \"" + table.name() + "\"" +
                Where::sql(wheres_,phase) + 
                By::sql(bys_,phase) + 
                Having::sql(havings_,phase) + 
                Order::sql(orders_,phase) + 
                LimitOffset::sql(limit_,offset_);
    }
    
    Datatable execute(const Datatable& table, unsigned short int phase){
        std::string s = sql(table,phase);
        return table.select(s);
    }
};


class Modifier : public Element {
public:

    virtual Elements values(void) const {
        return {};
    };

    virtual Bys bys(void) const {
        return {};
    };
};

//! @name Combiners
//! @brief Classes that combine levels of a categorical variable
//!
//! Often it is useful to combine categories of a categorical variable for a data summary.
//! A common instance is creating an "other" category. For example, you might want to obtain
//! sums for the top 10 levels and for all other categories combined. This is what Combiners do.
//! @{

class Combiner : public Modifier {
protected:
    
    //! @brief Element that is the subject of this combiner.
    By* by_;

public:

    //! @brief
    Combiner(Element* element){
        By* by = dynamic_cast<By*>(element);
        if(by) by_ = by;
        else by_ = new By(element);
    }
    
    virtual Bys bys(void) const {
        return {by_};
    };

    //! @brief
    //! @param datatable
    //!
    //! Combiners set values in corresponding columns to "<other>"
    virtual void combine(const Datatable& datatable) const = 0;
};

class Top : public Combiner {
protected:

    //! @brief
    Aggregate* aggregate_;
    
    //! @brief
    unsigned int number_;

public:

    //! @brief
    //! @param by
    //! @param element
    //! @param num
    Top(Element* by, Element* element,const unsigned int& number=10):
        Combiner(by),
        number_(number){
        Aggregate* aggregate = dynamic_cast<Aggregate*>(element);
        if(aggregate) aggregate_ = aggregate;
        else aggregate_ = new Aggregate("sum",element);
    }

    virtual std::string dql(void) const {
        return "top(" + by_->dql() + "," + aggregate_->dql() + "," + boost::lexical_cast<std::string>(number_) + ")";
    }
    
    virtual Elements values(void) const {
        return {
            aggregate_
        };
    }

    virtual void combine(const Datatable& datatable) const {
        //Determine the top levels
        std::stringstream sql;
        std::string subject = '"'+by_->id()+'"';
        std::string table = '"'+datatable.name()+'"';
        sql <<"UPDATE "<<table<<" SET "<<subject<<" = '<other>' WHERE "<<subject<<" NOT IN ("
                <<"SELECT "<<subject<<" FROM "<<table<<" GROUP BY "<<subject<<" ORDER BY "<<aggregate_->sql(2)<<" DESC LIMIT "<<number_
            <<")";
        datatable.execute(sql.str());
    }

};

class Margin : public Modifier {
protected:

    //! @brief The subject of this margin
    //!
    //! The summary will be repeated for each category in this Element
    By* by_;

public:

    //! @brief Construct a Margin
    //! @param subject The subject Element
    //!
    //! The subject element can be omitted in which case the margin is calculated over all
    //! rows in the supplied Datatable (see calculate())
    Margin(Element* element = 0){
        if(element){
            By* by = dynamic_cast<By*>(element);
            if(by) by_ = by;
            else by_ = new By(element);
        } else {
            by_ = 0;
        }
    }
    
    virtual std::string dql(void) const {
        std::string dql = "margin(";
        if(by_) dql += by_->dql();
        return dql + ")";
    }

    virtual Bys bys(void) const {
        if(by_) return {by_};
        else return {};
    }

    void calculate(const Datatable& from, const Datatable& to, const Elements& values, const Bys& bys) const {
        //Create a list of columns for a SQL select statement
        //It is necessary to have the same number of colums as in the to table but "fill in" some of those columns
        //with "<all>"
        Elements columns;
        for(const By* by : bys){
            if(by==by_) columns.push_back(by);
            else columns.push_back(new Constant<std::string>("<all>"));
        };
        //Append the list of values to the end of the columns list
        columns.insert(columns.end(),values.begin(),values.end());
        
        // Execute the SQL select
        Datatable alls = Select().columns(columns).by(this->bys()).execute(from,2);
        // Append the alls to the "to" table
        to.append(alls);
    }
};


//! @name Adjusters
//! @{

class Adjuster : public Modifier {
protected:

    //! @brief Value for which a proportion will be calculated 
    Element* value_;
    
    //! @brief Categorical variables for which a proportion will be calculated for each category
    Bys bys_;

public:

    Adjuster(Element* value):
        value_(value){
    }

    virtual std::string dql_name(void) const = 0;
    
    virtual std::string dql(void) const {
        std::string dql = dql_name() + "(" + value_->dql();
        for(const By* by : bys_) dql += "," + by->dql();
        return dql + ")";
    }

    virtual Elements values(void) const {
        return {value_};
    }

    virtual Bys bys(void) const {
        return bys_;
    }
    
    Adjuster& bys_append(const Element* element){
        const By* by = dynamic_cast<const By*>(element);
        if(!by) by = new By(element);
        bys_.push_back(by);
        return *this;
    }

    template<class... Elements>
    Adjuster& bys_append(const Element* element, const Elements*... elements){
        bys_append(element);
        bys_append(elements...);
        return *this;
    }

    //! @brief 
    //! @param final
    virtual void adjust(const Datatable& final) const = 0;
};

class Proportion : public Adjuster {
public:

    Proportion(Element* value):
        Adjuster(value){
    }

    virtual std::string dql_name(void) const {
        return "prop";
    }

    virtual void adjust(const Datatable& final) const {
        //Create
        Aggregate* sum = new Aggregate("sum",value_);
        
        // Calculate sums of values
        Elements columns;
        columns.insert(columns.begin(),bys_.begin(),bys_.end());
        columns.push_back(sum);
        //Execute with which==3 so that correct column names are used
        Datatable sums = Select()
            .columns(columns)
            .by(bys_)
            .execute(final,3);
            
        // Create ids for referencing columns in tables
        std::string value_id = value_->id();
        std::string sum_id = sum->id();
        // Create a list of Bys for join
        std::string bys;
        for(const By* by : bys_) bys += "\"" + by->id() + "\",";
        bys.erase(bys.end()-1,bys.end());
        // Create a name for the temporary table
        std::string temp_name = "stencila_"+boost::lexical_cast<std::string>(Hash())+"_temp";
        
        // Create a temporary table of calculated values, one value for each row in final table
        final.execute("CREATE TEMPORARY TABLE \"" + temp_name + "\" AS SELECT * FROM (SELECT \"" +
            value_id + "\"/\"" + sum_id + "\" AS \"_calc_\" FROM \"" + final.name() + "\" LEFT JOIN \"" + sums.name() + "\" USING(" + bys + "))");
        // Update values in final table using rowids in a subquery to join
        final.execute("UPDATE \"" + final.name() +"\" SET \"" + value_id +
            "\"=(SELECT \"_calc_\" FROM \"" + temp_name +"\" WHERE \"" + final.name() +"\".rowid==\"" + temp_name + "\".rowid)");
        
        //Cleanup
        delete sum;
    };
};

//! @todo class Centre : subtract mean
//! @todo class Scale : divide by geometric mean
//! @todo class Norm : adjust so mean is equal to zero and standard deviation is equal to 1

//! @}

//! @name Reshapers
//! @{

class Reshaper : public Element {
    //! @todo Implement
};

//! @todo class Column : specify column for cross-table
//! @todo class Row : specify row for cross-table
//! @todo claa Melt : puts column names into a row so that there is just one "value" column

//! @}


//! @class Dataquery
//! @todo Document fully
class Dataquery {
private:

    //! @brief 
    std::vector<Element*> elements_;
    
    //! @brief 
    std::string from_;

    //! @brief 
    bool compiled_;

    //! @brief 
    bool distinct_;
    
    //! @brief 
    Elements values_;
    
    //! @brief 
    Wheres wheres_;
    
    //! @brief 
    Bys bys_;
    
    //! @brief 
    Havings havings_;
    
    //! @brief 
    Orders orders_;
    
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
            
            for(Element* element: elements_){
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
                    combiners_.push_back(combiner);
                    Elements values = combiner->values();
                    values_.insert(values_.end(),values.begin(),values.end());
                    Bys bys = combiner->bys();
                    bys_.insert(bys_.end(),bys.begin(),bys.end());
                }
                else if(Margin* margin = dynamic_cast<Margin*>(element)){
                    margins_.push_back(margin);
                    Bys bys = margin->bys();
                    bys_.insert(bys_.end(),bys.begin(),bys.end());
                }
                else if(Adjuster* adjuster = dynamic_cast<Adjuster*>(element)){
                    adjusters_.push_back(adjuster);
                    Elements values = adjuster->values();
                    values_.insert(values_.end(),values.begin(),values.end());
                    Bys bys = adjuster->bys();
                    bys_.insert(bys_.end(),bys.begin(),bys.end());
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
    
public:

    //! @brief Execute this Dataquery on a Datatable
    //! @param table The table to execute this query on
    //! @return A datatable produced by this query
    Datatable execute(const Datatable& table){
        // Compile this query
        compile();
        
        Datatable result;
        
        // Define a list of columns for result which will have any Bys prepended
        Elements columns = values_;
        columns.insert(columns.begin(),bys_.begin(),bys_.end());

        if(combiners_.size()==0 and margins_.size()==0 and adjusters_.size()==0){
            // Select data
            // Since their are no modifiers do a single pass
            result = Select().distinct(distinct_).columns(columns).where(wheres_)
                .by(bys_).having(havings_).order(orders_).limit(limit_).offset(offset_)
                .execute(table,0);
        } else {
            // 1. Select data
            // As as a first pass, obtain the necessary columns applying any wheres and bys and using which==1
            // Execute with cache reuse, but no caching because combiners will modify the data in it
            Datatable first = Select().columns(columns).where(wheres_).by(bys_).execute(table,1);
            
            // 2. Apply combiners
            for(const Combiner* combiner : combiners_) {
                combiner->combine(first);
            }
            if(combiners_.size()) first.modified();
            
            // 3. Value calculations
            Datatable second = Select().columns(columns).by(bys_).execute(first,2);
            
            // 4. Margin calculations
            for(const Margin* margin : margins_){
                margin->calculate(first,second,values_,bys_);
            }
            if(margins_.size()) second.modified();
            
            // 5. Apply adjusters
            for(const Adjuster* adjuster : adjusters_){
                adjuster->adjust(second);
            }
            if(adjusters_.size()) second.modified();
            
            // 6. Reshape
            //! @todo Implement Reshapers
            
            // 7. Execute phase 10 select to obtain the desired column names for the table
            result = Select().columns(columns).execute(second,10);
        }
        
        return result;
    }
};

}
