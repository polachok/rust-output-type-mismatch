pub trait Reader<'a>: Sized {
    fn get(bytes: &'a [u8]) -> Self;
}

pub trait Owned<'a> {
    type Reader: Reader<'a>;
}

pub trait ConsumerService {
    type Item: for<'x> Owned<'x>;

    fn call(_item: <Self::Item as Owned>::Reader) {
        unimplemented!()
    }
}

struct M;
struct MReader<'a>(&'a [u8]);

impl<'a> Owned<'a> for M {
    type Reader = MReader<'a>;
}

impl<'a> Reader<'a> for MReader<'a> {
    fn get(bytes: &'a [u8]) -> Self {
        MReader(bytes)
    }
}

struct My;

impl ConsumerService for My {
	type Item = M;
}

fn consume_loop<T, M, P>(
    process: P,
) where
    T: ConsumerService + 'static,
    M: for<'x> Owned<'x>,
    P: Fn(Vec<<M as Owned>::Reader>) -> bool + Sync + Send + 'static,
{

    let readers: Vec<M::Reader> = unsafe { std::mem::uninitialized() };
    process(readers);
}

pub fn t<S: ConsumerService + 'static>(s: S) {
     consume_loop::<S, S::Item, _>(
        move |batch| {
            for item in batch {
                S::call(item);
            }
            false
        },
    )
}

fn main() {
    t(My);
}