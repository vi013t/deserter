use deserter::{load, loadable};

#[cfg(test)]
fn test() {

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

    let value = load!(   
        Person {
            name = "hi",
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
}
