---
# Test using `edit-code-chunk` in a document with no other content
---

%% @edit-code-chunk add comments
%>

```r exec
data <- read.csv('data.csv')
plot(height~age, data)
```

%%


%% @edit-code-chunk simplify
%>

```r exec
data <- read.table(
    'data.csv',
    header = TRUE,
    sep = ','
)
```

%%


%% @edit-code-chunk color points by species
%>

```r exec
ggplot(data, aes(x=age, y=height)) + geom_point()
```

%%
