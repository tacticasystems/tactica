# Tactica

The milsim unit management platform.

## Build

```
mise run build    # build the runtime components
mise run test     # run all tests
mise run check    # run all CI steps
```

`check` needs Docker to run the [`testcontainers`](https://testcontainers.com/)
fixtures.

CI is in Forgejo Actions, in `.forgejo/workflows/`.
