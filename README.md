# Defunctionalize

Defunctionalization as a proc macro for Rust.

Defunctionalization is a technique by which higher-order functions are eliminated, and
replaced instead by a first-order function.

Though this reduces flexibility somewhat, there are certain benefits. In particular,
it allows serialization of behaviour.

## Features

*   Defunctionalize all public functions in a module into an enum.
*   Extra parameters in the method get moved to the enum.
*   Can apply `derive` to the module to derive for the resulting enum.

## Usage

1.  Apply the `defunctionalize` attribute to a module. This attribute takes a function signature,
    which is the signature of the functions you intend to defunctionalize. This signature requires
    named arguments, and supports generics.

    Typically, the name of the generated enum type is computed from the name of the module, but by
    adding a name in this signature, that name is used instead. Note that the name is *not* converted
    to CamelCase automatically in this case.

    ```rust
    // Basic usage
    #[defunctionalize(fn(lhs: usize, rhs: usize) -> usize)]
    mod defunc_a {}

    // Generics are supported
    #[defunctionalize(fn<T: std::ops::Add>(lhs: T, rhs: T) -> T::Output)]
    mod defunc_b {}

    // The function name will override the module name
    #[defunctionalize(fn DefuncC(lhs: usize, rhs: usize) -> usize)]
    mod hello {} // The generated item will be `enum DefuncC { ... }`

    // Where clauses are supported too
    #[defunctionalize(fn<T>(lhs: T, rhs: T) -> T::Output where T: Add)]
    mod defunc_d {}
    ```

2.  You may apply the `derive` attribute to this module as well. The syntax is the same as usual,
    and the traits will be derived on the generated enum. The usual restrictions will apply for the
    types of the enum cases' fields.

3.  Define `pub` functions in this module. They will get converted to enum cases. Non-`pub` functions
    may be defined as helpers, but will not be added as enum cases.

    These functions must have at least the signature defined in the `defunctionalize` attribute, but
    may also have extra arguments *before* the listed ones. The return type must match.

    The name of the function is converted to CamelCase to become the name of the enum case.

    ```rust
    #[defunctionalize(fn(lhs: usize, rhs: usize) -> usize)]
    mod defunc_a {
        // Helper won't get defunctionalized
        fn helper(s: String) -> usize { s.len() }

        // This signature matches
        pub fn add(lhs: usize, rhs: usize) -> usize { lhs + rhs }

        // Extra parameters at the front get converted to enum fields
        pub fn add_plus_n(n: usize, lhs: usize, rhs: usize) -> usize { lhs + rhs }

        // This is not supported: the signature is wrong
        pub fn bad(x: String, y: usize) -> &'static str { "what" }
    }
    ```

## Examples

The most basic usage is as follows:

```rust
use defunctionalize::defunctionalize;

// Apply the defunctionalize attribute to a module. This attribute takes one parameter
// which is the signature of the generated `call` method.
//
// In this example, we are doing a mathematical operation, so our defunctionalized
// function takes two integers, and returns the result of the operation.
#[defunctionalize(fn(x: u32, y: u32) -> u32)]
pub mod operation {
    // All public functions in this module are then defunctionalized. Private functions
    // do not get defunctionalized, but they may be used internally.
    pub fn add(x: u32, y: u32) -> u32 { x + y }
    pub fn sub(x: u32, y: u32) -> u32 { x - y }
    pub fn mult(x: u32, y: u32) -> u32 { x * y }
    pub fn div(x: u32, y: u32) -> u32 { x / y }
    pub fn rem(x: u32, y: u32) -> u32 { x % y }
}

// The above module will be compiled into an enum, with a case for each
// public function:
//
// pub enum Operation {
//     Add,
//     Sub,
//     Mult,
//     Div,
//     Rem,
// }
//
// The name of the module is converted to CamelCase to become the name of the enum.
// Similarly, the name of the functions are converted to CamelCase to become the case
// names

// Values of the resulting enum can then be passed to functions. They contain a `call`
// method, which uses the signature provided in the defunctionalize attribute, which
// calls the corresponding method
fn perform_operation(operation: Operation) -> u32 {
    operation.call(6, 7)
}

// And here we use the defunctionalized module to compute 6 * 7 = 42!
fn main() {
    assert_eq!(42, perform_operation(Operation::Mult));
}
```

This, of course, is not the entire functionality of this technique! Here, we're going
to expand on the above to demonstrate two more useful features:

```rust
use defunctionalize::defunctionalize;

// First, notice that the function signature here has had one argument removed!
#[defunctionalize(fn(rhs: u32) -> u32)]
// Next, we put some derives *on the module*. These will get moved to the enum. This
// is most useful for serializing your defunctionalized functions.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub mod operation {
    // Back to the first point now, notice that these functions have not changed! Where
    // do they get the extra parameter though?
    pub fn add(x: u32, y: u32) -> u32 { x + y }
    pub fn sub(x: u32, y: u32) -> u32 { x - y }
    pub fn mult(x: u32, y: u32) -> u32 { x * y }
    pub fn div(x: u32, y: u32) -> u32 { x / y }
    pub fn rem(x: u32, y: u32) -> u32 { x % y }
}

// The extra parameter got moved to the enum! Now we can essentially serialize an
// entire function call.
//
// #[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub enum Operation {
//     Add(u32),
//     Sub(u32),
//     Mult(u32),
//     Div(u32),
//     Rem(u32),
// }

// Now we only need to provide one parameter to the `call` method:
fn perform_operation(operation: Operation) -> u32 {
    operation.call(7)
}

// And here we use the defunctionalized module to compute 49 - 7 = 42!
fn main() {
    assert_eq!(42, perform_operation(Operation::Sub(49)));
}
```
