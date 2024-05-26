# Deserter

A set of two procedural macros for initializing structs with a JavaScript-like syntax. In other words, **deser**ializing data with compile-time checking into struct values.

## Usage

The `#[loadable]` attribute can be added on structs which can be loaded, and the `load!` macro can be used to initialize a loadable struct. All fields which are themselves structs must also be marked `#[loadable]`:

```rust
use deserter::{load, loadable};

#[loadable]
struct ZipCode {
	digits: u32,
}

#[loadable]
struct Address {
	house: u32,
	street: &'static str,
	city: &'static str,
	zip_code: ZipCode,
}

#[loadable]
struct Person {
	name: &'static str,
	age: u32,
	address: Address,
}

fn example() {
	let john = load!(   
		Person {
			name = "john",
			age = 30,
			address = {
				house = 101,
				street = "Main Street",
				city = "New York",
				zip_code = {
					digits = 100200
				}
			}
		}
	);

	// do things with john, it is a `Person`.
}
```