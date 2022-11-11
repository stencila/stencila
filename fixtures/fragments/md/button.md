A button #[do_it] and another with a label specified #[go]{label="Go for it"}

This code chunk will run when either of the above buttons is clicked,

```python exec
f"Do it was last clicked at {do_it}. Go for it was last clicked at {go}."
```

Buttons can have a condition which must evaluate to a truthy value for the button to be enabled e.g.

#[maybe]`1<2`

The programming language of the expression can be specified e.g.

#[sometimes]`sum(x)>10`{python}
