An example article mainly intended for testing that number parameters can be set in different kernels.

This number parameter `a`: &[a]{num min=0 max=100 mult=0.1} should be multiplied by $\pi$ in the following `CodeChunk`s using different kernels:

```calc exec
a * 3.1415
```

```js exec
a * Math.PI
```

```py exec
import math
a * math.pi
```

```r exec
unbox(a * pi)
```
