extern crate capnp;
mod consumer;

use consumer::*;

struct My;

impl ConsumerService for My {
	type Item = capnp::any_pointer::Owned;
}

impl BatchConsumerService for My {
	fn call_batch(&mut self, items: Vec<capnp::any_pointer::Reader>) {
        for item in items {
            //println!("{:?}", item);
        }
	}
}

fn main() {
    Consumer::new().batch(|| My);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
