```quickjs exec
// A sleep function which can be called at top level
// without using await (which is not supported in QuickJS)
function sleep(seconds) {
    const startTime = new Date().getTime();
    let currentTime = null;
    do {
        currentTime = new Date().getTime();
    } while (currentTime - startTime < seconds * 1000);
}

sleep(1)
```

```quickjs exec
sleep(2)
```

```quickjs exec
sleep(4)
```

```quickjs exec
sleep(8)
```
