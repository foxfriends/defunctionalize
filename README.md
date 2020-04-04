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
