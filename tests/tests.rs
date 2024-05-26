use deserter::{load, loadable};

#[cfg(test)]
pub fn test() {
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
}
