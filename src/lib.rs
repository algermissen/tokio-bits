//! This create is a collection of modules useful for developing
//! Tokio based applications.
extern crate futures;

pub mod ops;
pub mod stream;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
