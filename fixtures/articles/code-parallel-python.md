This fixture is focussed on testing scheduling of parallel execution plans with Python.

Stage 1, a single setup chunk:

```python exec
from time import sleep
from datetime import datetime

def show_time(curr, prev_time, prev):
    return f"Chunk {curr} succeeded at {datetime.now()}, {datetime.now()-prev_time} after chunk {prev}"

sleep(1)
chunk1 = datetime.now()
print(f"Chunk 1 succeeded at {chunk1}")
```

Stage 2, three code chunks that should start at the same time and execute in parallel but with different durations.

```python exec
sleep(1)
show_time(2, chunk1, 1)
```

```python exec
sleep(2)
show_time(3, chunk1, 1)
```

```python exec
sleep(3)
show_time(4, chunk1, 1)
```

Stage 3, should begin after chunks 2-4 have finished.

```python exec
sleep(1)
chunk5 = datetime.now()
show_time(5, chunk1, 1)
```

Stage 4, three chunks that should run in parallel after chunk 5 and finish about 1 second after it.

```python exec
sleep(1)
show_time(6, chunk5, 5)
```

```python exec
sleep(1)
show_time(7, chunk5, 5)
```

```python exec
sleep(1)
show_time(8, chunk5, 5)
```
