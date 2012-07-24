#' A convienience function for calling C++ functions
call_ = function(symbol,...){
	.Call(symbol,...,package='stencila')
}

object_ = function(object){
      if(typeof(object)=='externalptr'){
        tag = call_('tag',object)
        object = new(tag,created=TRUE,pointer=object)
        return(object)
      }
      return(object)
}

class_ = function(class_name){
  
  setClass(
      class_name,
      representation=representation(
        created = 'logical',
        pointer = 'externalptr'
      ),
      prototype=prototype(
        created = FALSE
      )     
  )

#A constructor object which allows for syntax like
#	instance = Class()
#in addition to the usual
#	instance = new("Class")
  assign(class_name,function(...){
	return(new(class_name,...))
  },pos=1)
  
  setMethod('initialize',class_name,eval(substitute(function(.Object,created=FALSE,pointer=NULL,...){
    if(!created){
      .Object@pointer = call_(paste(class_name,'new',sep='_'),...)
      .Object@created = TRUE
    } else {
      .Object@pointer = pointer
      .Object@created = TRUE
    }
    return(.Object)
  },list(class_name=class_name))))
  
  setMethod('$',class_name,eval(substitute(function(x,name){
    function(...) {
      result = call_(paste(class_name,name,sep='_'),x@pointer,...)
      return(object_(result))
    }
  },list(class_name=class_name))))
  
  NULL
}

class_('Datacursor')

class_('Dataset')


#Column class allows for converting operators in following
# method. BUT that might not be necessary
setClass('Column',contains='character')
"==.Column" = function(a,b)paste(a,"==",b)
"<.Column" = function(a,b)paste(a,"<",b)

class_('Datatable')
setMethod('[',
	signature(x='Datatable'),
	function(x,i,j,...){
		#Dispatch needs to be done here rather than using several alternative
		#signatures in setMethod dispatch. That is because if the latter is used
		#then evaluation of arguments is done in the parent frame and expressions
		#such as "by(year)" fail
		#	i='missing',j='numeric': get column(s)
		#	i='missing',j='character' : get column(s)
		#	i='numeric',j='missing' : get row(s)
		#	i='numeric',j='numeric' : get values(s)
		#	i='numeric',j='character' : get values(s)
		
		#Record the call, removing first (name of function ('[')) and second ('x') arguments which are
		#not needed
		args = as.list(match.call()[-c(1,2)])
		
		#Iterate through args to determine type and what to do next. This needs to 
		#be done with substitute with parent frame
		
		#If arg[1] is numeric then restrict the result to that set of rows, regardless of the other arguments
		#If an arg is character then select that column
		rows = -1
		cols = -1
		directives = NULL
		
		names = names(args)
		for(index in 1:length(args)){
			name = names[[index]]
			arg = args[[name]]
			arg = substitute(arg)
			directive = paste(name,":",mode(arg),sep='')
			if(mode(arg)=='numeric'){
				if(name=='i') rows = eval(arg)
				else if(name=='j') cols = eval(arg)
				else {
					directive = paste(directive," const(",arg,")",sep='')
				}
			}
			else if(mode(arg)=='call'){
				func = arg[[1]]
				directive = paste(directive,":",func,sep='')
			}
			
			#See subset.data.frame for how we can evaluate each argument
			#with the parent frame as a "fallback" for symbols not in the database
      dataset_names = list(
        #Needs to initialise for each of the names in the database
        year = new("Column","year"),
        region = new("Column","region"),
        sales = new("Column","sales"),
        
        by = function(.) paste("by(",.,")",sep=""),
        sum = function(.) paste("sum(",.,")",sep=""),
        where = function(.) paste("where(",.,")",sep="")
      )
			directive = eval(arg,dataset_names,parent.frame())
      
			directives = c(directives,directive)
		}
		

		
		list(args=args,rows=rows,cols=cols,directives=directives)
	}
)

#methods(class='data.frame')
#	head & tail
#	subset
#	merge
#	summary
dim.Datatable = function(self) self$dimensions()
plot.Datatable = function(self) plot(self$dataframe())
as.data.frame.Datatable = function(self) self$dataframe()


