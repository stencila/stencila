# Issues - Iteration II

Goal: being able to observe a range of cells and get notified when a cell has been analysed or evaluated.

## Registration

```
let observer = engine.observeCellRange('C1:B10', {
  'analysed': (cell) => {},
  'evaluated': (cell) => {}
})
```

## Events

- `analysed`: called when a cell has been analysed
- `evaluated`: called when a cell has been evaluated

## Callback

```
/*
  @param {Cell} cell the Cell instance that has been updated.
 */
function(cell) {}
```

## Dispose

```
observer.dispose()
```
