This fixture is focussed on testing scheduling of parallel execution plans.

Stage 1, a single setup chunk:

```python exec
from time import sleep
from datetime import datetime

sleep(1)
chunk1 = datetime.now()
print(f"Chunk 1 succeeded at {chunk1}")
```

Stage 2, four code chunks that should get scheduled at the same time and execute in parallel but with different durations. Chunk 5 should begin when chunk 1 is finished and before any of the other chunks in this stage.

```python exec
sleep(1)
print(f"Chunk 2 succeeded at {datetime.now()} ({datetime.now()-chunk1} after chunk 1)")
```

```python exec
sleep(2)
print(f"Chunk 3 succeeded at {datetime.now()} ({datetime.now()-chunk1} after chunk 1)")
```

```python exec
sleep(3)
print(f"Chunk 4 succeeded at {datetime.now()} ({datetime.now()-chunk1} after chunk 1)")
```

```python exec
sleep(1)
chunk5 = datetime.now()
print(f"Chunk 5 succeeded at {chunk5} ({chunk5-chunk1} after chunk 1)")
```

Stage 3, starts after chunk 5 is finished

```python exec
sleep(1)
print(f"Chunk 6 succeeded at {datetime.now()} ({datetime.now()-chunk5} after chunk 5)")
```

```python exec
sleep(2)
print(f"Chunk 7 succeeded at {datetime.now()} ({datetime.now()-chunk5} after chunk 5)")
```

```python exec
sleep(3)
print(f"Chunk 8 succeeded at {datetime.now()} ({datetime.now()-chunk5} after chunk 5)")
```
