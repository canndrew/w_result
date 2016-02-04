//! Defines `WResult`, a result type that carries warnings.
//!
//! Sometimes it may be possible for an operation to proceed despite encountering errors. In these
//! cases, the caller may need to know about the errors that occured. `WResult` is similar to
//! `Result` except that the ok variant carries a vector of accumulated warnings. It comes with
//! methods for converting to a `Result` by discarding or logging the warnings or treating them as
//! errors.
//! 

#[macro_use]
extern crate log;

use std::fmt;
use std::iter::FromIterator;
use self::WResult::*;

/// A result type that carries warnings.
#[must_use]
pub enum WResult<T, W, E> {
    /// Contains the success value along with any accumulated warnings.
    WOk(T, Vec<W>),
    /// Contains the error value.
    WErr(E),
}

impl<T, W, E> WResult<T, W, E> {
    /// Returns true if this `WResult` is `WOk`
    pub fn is_ok(&self) -> bool {
        match *self {
            WOk(_, _) => true,
            WErr(_) => false,
        }
    }

    /// Returns true if this `WResult` is `WErr`
    pub fn is_err(&self) -> bool {
        match *self {
            WOk(_, _) => true,
            WErr(_) => false,
        }
    }

    /// Returns true if this `WResult` is `WErr` or if it is `WOk` but contains warnings.
    pub fn is_warn_or_err(&self) -> bool {
        match *self {
            WOk(_, ref ws) => ws.len() > 0,
            WErr(_) => true,
        }
    }

    /// Converts this `WResult` to an `Option` by taking the taking the `WOk` value or mapping
    /// `WErr` to `None`. Any warnings are discarded.
    pub fn ok_discard(self) -> Option<T> {
        match self {
            WOk(t, _) => Some(t),
            WErr(_) => None,
        }
    }

    /// Converts this `WResult` to an `Option` by taking the `WErr` variant or mapping `WOk` to
    /// `None`.
    pub fn err(self) -> Option<E> {
        match self {
            WOk(_, _) => None,
            WErr(e) => Some(e),
        }
    }

    /// Converts this `WResult` to an `Option` by taking the `WOk` variant or mapping `WErr` to
    /// `None`. This function is similar to `ok_discard` except if there are any warnings then they
    /// are treated as errors and this function returns `None`.
    pub fn ok_werr(self) -> Option<T> {
        match self {
            WOk(t, ws) => match ws.len() {
                0 => Some(t),
                _ => None,
            },
            WErr(_) => None,
        }
    }

    /// Map the `WOk` value of this `WResult`, if any.
    pub fn map<U, F>(self, op: F) -> WResult<U, W, E>
        where F: FnOnce(T) -> U
    {
        match self {
            WOk(t, ws) => WOk(op(t), ws),
            WErr(e) => WErr(e),
        }
    }

    /// Map the `WErr` value of this `WResult`, if any.
    pub fn map_err<U, F>(self, op: F) -> WResult<T, W, U>
        where F: FnOnce(E) -> U
    {
        match self {
            WOk(t, ws) => WOk(t, ws),
            WErr(e) => WErr(op(e)),
        }
    }

    /// Map the warnings of this `WResult`.
    pub fn map_warnings<U, F>(self, op: F) -> WResult<T, U, E>
        where F: FnMut(W) -> U
    {
        match self {
            WOk(t, ws) => WOk(t, ws.into_iter().map(op).collect()),
            WErr(e) => WErr(e),
        }
    }

    /// If `self` is `WOk`, returns `res` with the warnings from `self` accumulated into the final
    /// result. Otherwise returns the `WErr` value of `self`.
    pub fn and<U>(self, res: WResult<U, W, E>) -> WResult<U, W, E> {
        match self {
            WOk(_, mut ws0) => match res {
                WOk(t, ws1) => {
                    ws0.extend(ws1);
                    WOk(t, ws0)
                }
                WErr(e) => WErr(e)
            },
            WErr(e) => WErr(e),
        }
    }

    /// If `self` is `WOk`, returns the result of applying `op` to `self`'s value and warnings.
    /// Otherwise returns the `WErr` value of `self`.
    pub fn and_then<U, V, F>(self, op: F) -> WResult<U, V, E>
        where F: FnOnce(T, Vec<W>) -> WResult<U, V, E>
    {
        match self {
            WOk(t, ws) => op(t, ws),
            WErr(e) => WErr(e),
        }
    }

    /// If `self` is `WOk` returns `self`. Otherwise returns `res`.
    pub fn or<U>(self, res: WResult<T, W, U>) -> WResult<T, W, U> {
        match self {
            WOk(t, ws) => WOk(t, ws),
            WErr(_) => res,
        }
    }

    /// If `self` is `WOk` returns `self`. Otherwise returns the result of applying `op` to
    /// `self`'s error value.
    pub fn or_else<U, F>(self, op: F) -> WResult<T, W, F>
        where F: FnOnce(E) -> WResult<T, W, F>
    {
        match self {
            WOk(t, ws) => WOk(t, ws),
            WErr(e) => op(e),
        }
    }

    /// Convert this `WResult<T, W, E>` to a `Result<T, E>`, discarding any errors. See also
    /// `result_log` for a version of this function that logs warnings.
    pub fn result_discard(self) -> Result<T, E> {
        match self {
            WOk(t, _) => Ok(t),
            WErr(e)   => Err(e),
        }
    }

    /// Convert this `WResult<T, W, E>` to a `Result<T, Result<W, E>>`. This is a way to convert
    /// from `WResult` to `Result`, treating warnings as errors but allowing `W` and `E` to be two
    /// different types. See also `result_werr` for when `W` and `E` are the same type. If there
    /// are multiple warnings the first is returned.
    pub fn result_werr_res(self) -> Result<T, Result<W, E>> {
        match self {
            WOk(t, mut ws) => {
                ws.truncate(1);
                match ws.pop() {
                    Some(w) => Err(Ok(w)),
                    None => Ok(t),
                }
            }
            WErr(e) => Err(Err(e)),
        }
    }

    /// If `self` is `WOk`, unwraps it discarding any warnings. Otherwise returns `optb`. See also
    /// `unwrap_log_or` for a version of this function that logs warnings.
    pub fn unwrap_discard_or(self, optb: T) -> T {
        match self {
            WOk(t, _) => t,
            WErr(_) => optb,
        }
    }

    /// If `self` is `WOk` and has no warnings, unwraps it. Otherwise returns `optb`.
    pub fn unwrap_or_werr(self, optb: T) -> T {
        match self {
            WOk(t, ws) => match ws.len() {
                0 => optb,
                _ => t,
            },
            WErr(_) => optb,
        }
    }

    /// If `self` is `WOk`, unwraps it discarding any warnings. Otherwise returns the result of
    /// applying `op` to `self`'s error value. See also `unwrap_log_or_else` for a version of this
    /// function that logs warnings.
    pub fn unwrap_discard_or_else<F>(self, op: F) -> T
        where F: FnOnce(E) -> T
    {
        match self {
            WOk(t, _) => t,
            WErr(e) => op(e),
        }
    }
}

impl<T, E> WResult<T, E, E> {
    /// Take the error value of this `WResult`, if any. Otherwise returns the first warning, if
    /// any. This function is the same as `WResult::err` except that warnings are treated as
    /// errors.
    pub fn err_werr(self) -> Option<E> {
        match self {
            WOk(_, mut ws) => {
                ws.truncate(1);
                ws.pop()
            },
            WErr(e) => Some(e),
        }
    }

    /// Convert this `WResult` to a `Result` but treat warnings as errors. If there are multiple
    /// warnings the first is returned.
    pub fn result_werr(self) -> Result<T, E> {
        match self {
            WOk(t, mut ws) => {
                ws.truncate(1);
                match ws.pop() {
                    Some(w) => Err(w),
                    None => Ok(t),
                }
            },
            WErr(e) => Err(e),
        }
    }

    /// If `self` is `WOk` and has no warnings then unwrap it. Otherwise return the result of
    /// applying `op` to `self`'s error or first warning.
    pub fn unwrap_or_else_werr<F>(self, op: F) -> T 
        where F: FnOnce(E) -> T
    {
        match self {
            WOk(t, mut ws) => {
                ws.truncate(1);
                match ws.pop() {
                    Some(w) => op(w),
                    None => t,
                }
            },
            WErr(e) => op(e),
        }
    }
}

impl<T, W, E> WResult<T, W, E>
    where W: fmt::Display
{
    /// Take the `WOk` value of `self`, if any. Warnings are logged using the `warn!` macro before
    /// being discarded.
    pub fn ok_log(self) -> Option<T> {
        match self {
            WOk(t, ws) => {
                for w in ws {
                    warn!("{}", w);
                }
                Some(t)
            },
            WErr(_) => None,
        }
    }

    /// Convert this `WResult<T, W, E>` to a `Result<T, E>`. Warnings are logged using the `warn!`
    /// macro before being discarded.
    pub fn result_log(self) -> Result<T, E> {
        match self {
            WOk(t, ws) => {
                for w in ws {
                    warn!("{}", w);
                }
                Ok(t)
            }
            WErr(e) => Err(e),
        }
    }

    /// If `self` is `WOk`, unwrap it and log any warnings using the `warn!` macro. Otherwise
    /// return `optb`.
    pub fn unwrap_log_or(self, optb: T) -> T {
        match self {
            WOk(t, ws) => {
                for w in ws {
                    warn!("{}", w);
                }
                t
            },
            WErr(_) => optb,
        }
    }

    /// If `self` is `WOk`, unwrap it and log any warnings using the `warn!` macro. Otherwise
    /// return the result of applying `op` to `self`'s error value.
    pub fn unwrap_log_or_else<F>(self, op: F) -> T
        where F: FnOnce(E) -> T
    {
        match self {
            WOk(t, ws) => {
                for w in ws {
                    warn!("{}", w);
                }
                t
            },
            WErr(e) => op(e),
        }
    }
}

impl<T, W, E> From<Result<T, E>> for WResult<T, W, E> {
    fn from(val: Result<T, E>) -> WResult<T, W, E> {
        match val {
            Ok(t) => WOk(t, Vec::new()),
            Err(e) => WErr(e),
        }
    }
}

impl<A, T, W, E> FromIterator<WResult<A, W, E>> for WResult<T, W, E>
    where T: FromIterator<A>
{
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=WResult<A, W, E>>
    {
        struct Adapter<Iter, W, E> {
            iter: Iter,
            warnings: Vec<W>,
            err: Option<E>,
        }

        impl<T, W, E, Iter: Iterator<Item=WResult<T, W, E>>> Iterator for Adapter<Iter, W, E> {
            type Item = T;

            fn next(&mut self) -> Option<T> {
                match self.iter.next() {
                    Some(WOk(t, ws)) => {
                        self.warnings.extend(ws);
                        Some(t)
                    },
                    Some(WErr(e)) => {
                        self.err = Some(e);
                        None
                    },
                    None => None,
                }
            }
        }

        let mut adapter = Adapter { iter: iter.into_iter(), warnings: Vec::new(), err: None };
        let t: T = FromIterator::from_iter(adapter.by_ref());
        
        match adapter.err {
            Some(e) => WErr(e),
            None => WOk(t, adapter.warnings),
        }
    }
}

