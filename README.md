# Tactica

The milsim unit management platform.

See [the unit API map](docs/api.md) for implemented read endpoints, access rules,
and the next endpoint batches.

## Build

```
mise run build    # build the runtime components
mise run test     # run all tests
mise run check    # run all CI steps
```

`check` needs Docker to run the [`testcontainers`](https://testcontainers.com/)
fixtures.

CI is in Forgejo Actions, in `.forgejo/workflows/`.
