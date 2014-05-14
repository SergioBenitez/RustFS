Rust Cheat Sheet V1
===================

Variable Declarations
---------------------

### Simple, immutable variable declaration

    let my_var = some_value;
    my_var = some_other_value; // not okay

### Mutable variable declaration

    let mut my_var = some_value;
    my_var = some_other_value; // okay

### Variable declaration with explicit type

    let my_var: SomeType = some_value;

### Type casting

    let my_other_var = my_var as some_other_type;

Type Aliases (not new types)
----------------------------

### Simple Alias

    type MyNewType = ExistingType;

### Tuple Type Alias

    type IntPair = (int, int);

Function Definition and Call
----------------------------

### Simple Function with 2 parameters and return value

    fn function_name(arg1: Type1, arg2: Type2) -> ReturnType {
      let my_var = some_value;
      ReturnType { field1: my_var }
    }

    let return_val = function_name(my_arg1, my_arg2);

### A Generic Function w/Closure Op and No Return Value

    fn apply<T>(values: &Vec<T>, op: |&T|) {
      for val in values.iter() {
        op(val);
      }
    }

    let my_vals = vec![1, 2, 3, 4]; // owned Vec<int>
    apply(&my_vals, |x: &int| { println!("{}", x) });

Control Structures
------------------

### Conditionals

    if this1 || (that1 && this2) {
      do_this;
    } else if this3 == that2 {
      something_else_yet;
    } else {
      oh_well;
    }

### While loop

    while some_condition {
      do_this;
    }

### Infinite Loop

    loop {
      if some_condition {
        break; // exit
      }
    }

### For Loop

    for c in iterator_implementing_object { //  See Iterator Protocol
      use(c);
    }

Data Structures
---------------

### Structs

    struct StructName {
      field1: Type,
      field2: Type2
    }

    let s = StructName {
      field1: value_1,
      field2: value_2
    }

### Generic Structs

    struct StructName<T> {
      field1: OtherObject<T>,
      field2: Type
    }

    let s = StructName {
      field1: OtherObject { field: thing_of_T },
      field2: value_2
    }

### Struct Tuples

    struct MyStruct(Type1, Type2, Type3);
    struct Meters(MyStruct);

    let s = MyStruct(val_of_type_1, val_of_type_2, val_of_type_3);
    let s2 = Meters(s);

### Constant Enums

    enum EnumName {
      Type1,
      Type2
    }

    let e = Type1;

### Constant Enums w/Discriminator Values

    enum EnumName {
      Type1 = 0xFF000,
      Type2 = 212
    }

    let e = Type1;

### Enums w/Struct Tuples

    enum EnumName {
      Type1(int, ~str),
      Type3(int),
      Type2
    }

    let e = Type3(34);

### Generic Enums w/Struct Tuples

    enum GenEnumName<T> {
      Type1(T, f32),
      Type3(int),
      Type2
    }

    enum EnumWithStr<'r> {
      Type4(&'r str),
    }

    let e = Type1("hello", 34); // T = &'static str
    let e = Type4("hello"); // 'r = 'static

### Enums w/Struct Variants (not fully supported)

    enum Shape {
      Circle { radius: int },
      Rectangle { length: int, width: int },
      Type3(int),
      Other
    }

### Tuples

    type WeirdTuple = (int, f32, &'static str);
    let s = (1, 2.34, "hello");
    let v: WeirdTuple = s;

Pattern Matching
----------------

### General Syntax

    match something {
      pattern1 => with_one_thing_no_block,
      pattern2 | pattern3 => {
        do_this;
        do_that;
      } // no comma needed when using blocks
      pattern3 if some_cond => do_that(),
      M..N => in_closed_range_M_N,
      x @ K..Q => x_is_in_K_Q(x),
      _ => if_all_else_fails
    }

### Destructing Tuple

    let s: (int, int);
    let weird_mult = match s {
      (123, 321) => 39483,
      (0, _) | (_, 0) => 0,
      (-1, y) | (y, -1) if y <= 0 => y,
      (x, y) => x * y
    }

### Matching Against and Destructing Enum

    enum Direction { N(int), S, E(int, (int, int)), W(~str), O(int, int, int) }
    let dir: Direction;
    match dir {
      N(x) => println!("up {}", x),
      S => println!("down"),
      E(_, (x, y)) => println!("right ({}, {})", x, y),
      W(ref s) => println!("left, {}", s),
      O(..) => println!("other")
    }

### Destructing Struct

    match some_struct {
      StructName { field1: pattern, other, .. } => something(pattern, other),
      StructName { .. } => something_else
    }

### Matching/Destructing Slices or Fixed-Size Vectors

    let numbers: &[int] = &[1, 2, 3];
    let score = match numbers {
      [] => 0,
      [a] => a * 10,
      [a, b] => a * 6 + b * 4,
      [a, b, c, ..rest] => a * 5 + b * 3 + c * 2 + rest.len() as int
    };

### In Variable Declerations

    let irrefutable_pattern = some_object;
    fn my_fn(irrefutable_pattern: Type, ...) { }


Traits
------
### Declaring a trait

    trait MyTrait {
      fn trait_requires_func(&self) -> WithRetType;

      fn trait_has_default_func_so_optional(&self) -> WithRetType {
        default_implementation();
      }
    }

### Implementing Trait on Struct

    impl MyTrait for MyStruct {
      fn trait_requires_func(&self) -> WithRetType {
        WithRetType { .. }
      }
    }

### Generic Trait

    trait TraitName<T> {
      fn trait_fn(val: T);
    }

    // example
    trait Stack<T> {
      fn push(val: &T);
      fn pop() -> Option(T);
    }
    
### Implementing Generic Trait

    impl Stack<int> for MyIntStack {
      ...
    }

    impl<T> Stack<T> for MyGenericStack<T> {
      ...
    }

### Trait Inheritance

    // Compiler will force implementors of Child to implement Super1 and Super2
    trait Child : Super1 Super2 {
      fn new_trait_fn(...) -> RetVal { ... }
    }

Methods
-------
### Implementing Methods on Base Type (Enum or Struct)

    impl TypeName {
      // With immutable reference to self
      fn instance_method(&self, arg1: Type1, arg2: Type2) -> ReturnVal {
        // self.field = something; // not allowed, self is immutable
        ReturnVal { ... }
      }

      // With immutable reference to self
      fn mut_instance_method(&mut self, arg1: Type1, arg2: Type2) {
        self.field = something; // okay
        ReturnVal { ... }
      }

      // Class methods simply don't take &self param
      fn class_method(arg1: Type1, arg2: Type2) -> TypeName {
        TypeName { ... }
      }
    }

    let obj = TypeName::class_method(arg1, arg2); // Calling class method.
    let some_val = obj.instance_method(a1, a2);  // Calling instance method.

### Implementing Traits (Any Type)
    
    impl TraitName for Type {
      fn trait_fn(trait_arg1: Type1, trait_arg2: Type2) -> TraitReturnType {
        TraitReturnType { }
      }
    }

    let t: Type;
    t.trait_fn(a1, a2);  // Called just like if methods were defined on type


Common Traits to Implement
--------------------------

### Iterator trait, for use in for loop (see iterator protocol)

    trait Iterator<T> {
      fn next(&mut self) -> Option<T>; // Required.
      fn size_hint(&self) -> (uint, Option<uint>); // Optional
    }

### Destructor (drop called on deallocation)

    trait Drop {
      fn drop(&mut self);
    }

### Add/Sub/Mul/Div/Rem (+-*/% operator overloading, called on self +-*/% rhs)

    pub trait Add/Sub/Mul/Div/Rem<RHS, Result> {
      fn add/sub/mul/div/rem(&self, rhs: &RHS) -> Result;
    }
    
### BitAnd/BitOr/BitXor (&|^ operator overloading, called on self &|^ rhs)

    pub trait BitAnd/BitOr/BitXor<RHS, Result> {
      fn bitand/bitor/bitxor(&self, rhs: &RHS) -> Result;
    }

### Shl/Shr (<< >> operator overloading, called on self << >> rhs)

    pub trait Shl/Shr<RHS, Result> {
      fn shl/shr(&self, rhs: &RHS) -> Result;
    }

### Neg/Not (-! unary operator overload, called on -!self)

    pub trait Neg/Not<Result> {
      fn neg/not(&self) -> Result;
    }

### Deref (* operator overload; * will dereference reference returned by deref)

    // Called when dereferencing immutable variable
    pub trait Deref<Result> {
      fn deref<'a>(&'a self) -> &'a Result;
    }

    // Called when dereferencing mutable variable
    pub trait DerefMut<Result> : Deref<Result> {
      fn deref_mut<'a>(&'a mut self) -> &'a mut Result;
    }
      
### Index ([] operator overloading, called on self[index])

    pub trait Index<Index, Result> {
      fn index(&self, index: &Index) -> Result;
    }

### Eq (==, != operator overloading; called on self == != other)
    
    pub trait Eq {
      fn eq(&self, other: &Self) -> bool;
      fn ne(&self, other: &Self) -> bool { ... has default ... }
    }

### Ord (<, >, <=, >=)
    
    pub trait Ord : Eq {
      fn lt(&self, other: &Self) -> bool;

      fn le(&self, other: &Self) -> bool { ... }
      fn gt(&self, other: &Self) -> bool { ... }
      fn ge(&self, other: &Self) -> bool { ... }
    } 

### Derivable
  * Eq (==, !=)
  * TotalEq (==, != where !(==) == (!==))
  * Ord (<, >, <=, >=)
  * TotalOrd (<, >, <=, >= where strict inverses are true)
  * Clone (clone)
  * Hash (hash function, used by hash map and others)
  * Rand (rand<R: Rng>(&mut R) -> Self, returns rand instance using RandNumGen)
  * Default (default() -> Self, returning default value for Type)
  * Zero (zero() -> Self, is_zero() -> bool, define additive identity)
  * FromPrimitive (from_i64/int/etc/() -> Option<Self>, converts from primitive)
  * Show (fmt(&self, &mut Formatter) -> Result, for printing)
  * Encodable (encode(), for serialization)
  * Decodable (decode(), for serialization)

To use:

    #[deriving(Trait1, Trait2)]
    struct StructName { ... }

    #[deriving(Rand, Show)]
    enum EnumName { ... }

    
Iterator Protocol
-----------------

### Iterator<T> Trait Functions to Implement

    trait Iterator<T> {
      fn next(&mut self) -> Option<T>; // Required.
      fn size_hint(&self) -> (uint, Option<uint>); // Optional
      // Return a lower bound and upper bound on the remaining length of the
      // iterator. The common use case for the estimate is pre-allocating space
      // to store the results
    }

### Example

    struct Repeater<T> {
      value: T,
      times: uint
    }

    impl<T: Clone> Iterator<T> for Repeater<T> {
      fn next(&mut self) -> Option<T> {
        if self.times > 0 {
          self.times -= 1;
          Some(self.value.clone()) 
        } else {
          None
        }
      }
    }

    fn repeat<T>(val: T, times: uint) -> Repeater<T> {
      Repeater { value: val, times: times }
    }

    for k in repeat("hello", 3) {
      println!("{}", k)
    }

Closures
--------

### Stack Closures

    // These close over and live on the stack.
    // They _refer_ to stack variables: they don't own them, thus, no moves.
    // Can only be used as arguments: cannot be stored outside variables.
    let x = |arg| { use(arg) };
    let y = |arg: InferredType, arg2: InferredType2| -> InferredReturn {
      use(arg1, arg2)
    };

    // A function taking in a stack closure
    fn do_twenty_give_last<O>(op: |int, int| -> O) -> O {
      for i in range(0, 19) { op(i, i); }
      op(20, 20)
    }

### Owned Closures

    // These things are weird and ill-defined at the moment, AFAIK.
    // Closed over-values are moved into the closure which take ownership.
    // They can only be executed once AFAIK.
    let x = proc(arg1: Type1, arg2: Type2) -> ReturnType { ... };

Crates and Modules
------------------

### Definitions and Explanation
Crates are the compile unit: they _are_ the program. Modules are namespaced
pieces of code. A file implicitly declares a module of the same name, but
modules can also be declared explicitly by using 'mod modname { mod_code }'. If
a file is named 'mod.rs' and it is in a folder named 'module_name', it defines a
module named 'module_name'.

To import code from another module, use the 'mod' keyword followed by the module
path: 'mod module::path' To access module code, use the module's path followed
by an item: 'mod::path::fn_name()'. Precompiled libraries can be imported by
using the 'extern crate' keywords: 'extern crate extra' => 'extra::some_item'.

To 'use' keyword can be used to alias module paths for easy use of module items.
Simply 'use mod::path::to::items' to use items as 'items::fn_item()'. Further,
items can be aliased as well: 'use mod::path::to::items::fn_item' means we can
do 'fn_item()'. A 'pub use' is a reexport of a module's items: the items become
part of the 'pub use'ing module's namespace and can be used by other modules
importing that module.

### Visibility and Example
Only items that are delcared 'pub' may be used outside of a module. See the
following for an example:

    // in farm.rs
    use chicken::Chicken;
    mod chicken;

    pub fn cool_chicken() -> Chicken;

    mod house {
      pub use chicken::Chicken;

      fn eat_chicken(chick: &Chicken) { ... }
      pub fn car_horn() { ... }
    }

    // in chicken/mod.rs
    pub struct Chicken { ... }

    // in main.rs
    mod farm;

    fn main() {
      let my_chick = farm::cool_chicken(); // okay, cool_chicken() is public
      // farm::house::eat_chicken(&my_chick); // not okay, eat_chicken is priv
      // let chick = chicken::Chicken { ... } // not okay, no 'mod chicken'
      let chick = farm::house::Chicken { ... } // okay, Chicken rexported
      farm::house::car_horn(); // okay, car_horn public
    }

### Order
The previous declarations must be used in the following order:

  1. extern crate
  2. use
  3. mod

Move and Copy Semantics
-----------------------
Like C++11, Rust distinguishes between L-Values and R-Values. Unlike C++11, Rust
also distinguishes between pure-valued values (copy values) and reference
owning or ownable values (owned values). It is in the distinction of copy-values
and owned-values that Rust's move and copy semantics come into play.

Types that contain owning pointers or values that implement a destructor via the
special trait Drop are owned values. All other types are copy values. 

When a value is used as an R-Value, the variable will either be moved or copied,
depending on its type. If a value is a copy value, assignments of any form to
for that value will create a shallow copy of the value. If a value is an owned
value, assignments of any form will move that value to the new assignment
yielding the previous assignment invalid.

Simple examples are as follows:

    let x = box 10; // x references box (owned value) with 10 in it, x owns box
    let y = x;      // box is moved: y now owns box, x is invalid

    let x = 10;     // x holds the value 10, a copy-value
    let y = x;      // y now also holds the value 10, x and y are both valid

    let x = box Thing { val: 10 }; // x references box (owned value), x owns box
    let y = x;                     // box is moved: y now owns box, x is invalid

    let x = Thing { val: 10 };     // x holds struct value
    let y = x;                     // value copied into y, y holds same value

Forms of Assignments:

  * let variable declaration
  * pattern matching assignment
  * function parameter assignments
  * structure/enum field assignments
  * anything that sets a variable anywhere

Since we may have a reference to an owned value that we wish to copy so that
another variable has another, different owning reference to the same underyling
value, Rust provides the 'clone' trait/method that can be implemented to copy
a value to a new owning box.

A simple example:

    let x = box 10;     // x owns box with 10 in it
    let y = x.clone();  // box not moved: y owns different box with same value

Sometimes we neither want to copy a value nor move the value, as in when using
an owned reference as a parameter to a function, we simply want to let another
piece of code read or modify the underyling value without taking ownership. For
this, Rust has a solution: the borrowed pointer. A borrowed-pointer is a
non-owning reference to any value, including owned values. There are immutable
and mutable variants. If a borrowed reference to a value is currently in scope,
the value cannot be moved since this would result in a dangling pointer. 

Simple Examples:

    let a = box 10; // a owns box with 10 in it
    
    {
      let b = &a;     // b is borrowing a's box, a still owns it
      let c = &a;     // c is borrowing a's box, a still owns it
      // let d = a;   // invalid since borrowed references are in scope 
    } // borrowed pointer now out of scope, can move a's box

    let mut d = a;    // a's box moved to d; a is invalid

    // Calling this function with a owned value will move the value to 'val'.
    fn bad_use_value<T: Show>(val: T) {
      println!("{}", val);
    }

    let q = box 10;   // q owns box with 10 in it
    bad_use_value(q); // box moved to 'val'. q is invalid

    // Calling this function with a owned value will borrow the value.
    fn good_use_value<T: Show>(val: &T) {
      println!("{}", val);
    }

    let q = box 10;    // q owns box with 10 in it
    good_use_value(q); // box lent to 'val' and returned once 'val' out of scope

Note that &mut is an owning reference itself: the single mutable borrowed
pointer may be moved to another owned:

    let a = box 10;   // a owns box with 10 in it
    let mut d = a;    // a's box moved to d; a is invalid

    {
      let e = &mut d; // e is mutable borrowed pointer to d's box
      let z = e;      // mutable borrowed pointer moves to z, e is invalid
      // cannot move d's box since borrowed reference is out
      // cannot borrow d's box since it's already mutably borrowed
    } // borrow's out of scope, back to d simply owning its box


Common Errors
-------------

### "cannot move out of dereference of & pointer"
In this error, you have a reference to an owned object, call it x, and are
trying to move the object to a new location, call it y, by dereferencing x
into y. That is:

    let x = &OwnedObject{};
    let y = *x;

The issue is that if the second operation were allowed, x would be left as a
pointer to the owned object, but y is now the owner, and therefore, if y were to
be dropped (ie, go out of scope), the owned object would be deallocated but x
would remain with a pointer to it: a dangling pointer.

#### Possible fixes:
Let y be a reference to the owned object as well:

    let x = &OwnedObject{};
    let y = x;

Move the owned object:

    let x = OwnedObject{};
    let y = x;

Copy the owned object:

    let x = &OwnedObject{};
    let z = x.clone();
