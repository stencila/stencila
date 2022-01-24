This fixture is focussed on testing scheduling of serial execution plans. The execution plan for this document should have four stages, scheduled to be run serially, but the fourth should not run because the third stage, which it depends upon, fails.

```bash exec
sleep 3
echo "Succeeded at $(date)."
```

```bash exec
# @requires cc-1
sleep 3
echo "Succeeded at $(date)."
```

```bash exec
# @requires cc-2
sleep 3
echo "Failed at $(date)."
echo "This chunk should show an error and the next should be scheduled but not run"
echo "Error to cancel next stage" >&2
```

```bash exec
# @requires cc-3
echo "This output should not show (unless it is explicitly run) and the chunk should show that a dependency failed."
```
