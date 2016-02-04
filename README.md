# w_result

This Rust crate defines `WResult`, a result type that carries warnings.

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
warnings alongside it's valuie. The advantages over just using
`Result<(T, Vec<W>), E>` are
 * Clarity.
 * Some `WResult` methods will automatically accumulate warnings from multiple
   `WResult`s (See for example `and` and the `FromIterator<WResult<..>>` impl).
 * `WResult<T, W, E>` comes with methods to convert it to a `Result<T, E>` by
   discarding or or logging the warnings or treating them as errors.

