# `Cell`



## Conversions

### XLSX

Use a special formula name and `@1`, `@2`,... placeholder syntax. Allows the opened excel files to be used with Stencila [custom functions](https://docs.microsoft.com/en-us/office/dev/add-ins/excel/custom-functions-overview)

For example, a cell in position `C1` which fits a linear model to the data in the first tem rows of columns `A` and `B`:

```
STENCILA.R("lm(@1~@2)", A1:A10, B1:B10)
```

```xml
<c r="C1" s="0" t="e">
	<f>STENCILA.R(&quot;lm(@1~@2)&quot;, A1:A10, B1:B10)</f>
	<v>#NAME?</v>
</c>
```

### ODT

See XLSX

```xml
<table:table-cell table:formula="of:=STENCILA.R(&quot;lm(@1~ @2)&quot;; [.A1:.A10]; [.B1:.B10])" calcext:value-type="error">
	<text:p>#NAME?</text:p>
</table:table-cell>
```
