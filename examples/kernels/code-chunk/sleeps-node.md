```nodejs exec
// A sleep function which can be called at top level
// without using await (which is not supported in
// Node.js's `vm` module)
function sleep(seconds) {
    const startTime = new Date().getTime();
    let currentTime = null;
    do {
        currentTime = new Date().getTime();
    } while (currentTime - startTime < seconds * 1000);
}

sleep(1)
```

```nodejs exec
sleep(2)
```

```nodejs exec
sleep(4)
```

```nodejs exec
sleep(8)
```
