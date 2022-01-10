1. Not yet run
2. Re-run required, semantics have changed
3. Re-run required, one or more dependencies have changed
4. Re-run required, one or more dependencies was out of sync with kernel
5. Scheduled
6. Running
7. Succeeded `executeEnded` ago in `executeDuration`
8. Failed `executeEnded` ago after `executeDuration`
9. Cancelled `executeEnded` ago after `executeDuration`

Here, `executeEnded` and `executeDuration` would be "humanized" e.g. "11s ago in 23s".

The pseudocode for which of theses states a `CodeChunk` or `CodeExpression` is in is roughly:

`executeStatus === undefined` => 1
`compileDigest[1] !== executeDigest[1]` => 2
`compileDigest[2] !== executeDigest[2]` => 3
`executeDigest[3] !== "0"` => 4
`executeStatus === "Scheduled"` => 5
`executeStatus === "Running"` => 6
`executeStatus === "Succeeded"` => 7
`executeStatus === "Failed"` => 8
`executeStatus === "Cancelled"` => 9
