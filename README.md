# Manual Executor

![Manual Executor](assets/visual.webp)

This is a manual executor for driving futures to readiness. You need to manually wake the futures! But of course what else dit you expect from an executor that is called manual-executor. Because of the manual nature the use cases of this executor are limited.

It was built to be used via ffi (also wasm). Imagine a library that wants to do a http-request. It has no way of doing it except via the host. So the host exposes a `fetch` function that does the http-request. This is all done asynchronously. How does the library know when the `fetch` function is done? That problem is solved with the manual-executor. The host simply calls the `wake` (or `wake_all` if you are a bit sloppy) function with the appropriate key.

## coverage

Install `grcov`, `llvm-tools`.

Like this (on arch)

```
pacman -S grcov
rustup component add llvm-tools
```

You might want to manually clean up the target directory from time to time to keep grcov form not erroring...
