pub trait DeFn<Input> {
    type Output;

    fn call(self, args: Input) -> Self::Output;
}

#[cfg(feature = "proc-macro")]
#[macro_use]
#[allow(unused_imports)]
extern crate defunctionalize_proc_macro;
#[cfg(feature = "proc-macro")]
#[doc(hidden)]
pub use defunctionalize_proc_macro::*;
