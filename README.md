# w_result

This Rust crate defines `WResult`, a result type that carries warnings.

[Documentation](http://canndrew.org/rust-doc/w_result)

Sometimes it may be possible for an operation to proceed despite encountering
errors. In these cases it's useful to notify the caller about these errors in
case they need to act on them. For this, functions can return a `WResult`.

```rust
enum WResult<T, W, E> {
    WOk(T, Vec<W>),
    WErr(e),
}
```

`WResult` is similar to `Result` except that the ok variant carries a vector of
warnings alongside it's value. The advantages over just using
`Result<(T, Vec<W>), E>` are
 * Clarity.
 * Some `WResult` methods will automatically accumulate warnings from multiple
   `WResult`s (See for example `and` and the `FromIterator<WResult<..>>` impl).

## Conversion to `Result`

Often, the only thing you'll want to do with a `WResult` is convert it to a
`Result`. `WResult` provides several methods for doing this that differ in how
they handle the warnings:

 * `WResult::result_log(self) -> Result<T, E>` converts to a `Result` by
   logging the warnings using the `log` crate.
 * `WResult::result_discard(self) -> Result<T, E>` converts to `Result` by
   discarding the warnings.
 * `WResult::result_werr(self) -> Result<T, E>` converts to `Result` by
   treating warnings as errors. If there are warnings, the first warning is
   returned as the error. This method only exists if the warning and error
   types are the same type. 
 * `WResult::result_werr_res(self) -> Result<T, Result<W, E>>` is similar to
   `result_werr` but allows the warning and error types to be two different
   types.

