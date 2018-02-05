use std;
use ::capnp;

fn consume_loop<T, M, P, I>(
    init: I,
    process: P,
) where
    T: ConsumerService + 'static,
    M: for<'x> capnp::traits::Owned<'x>,
    P: Fn(&mut T, Vec<<M as capnp::traits::Owned>::Reader>) -> bool + Sync + Send + 'static,
    I: Fn() -> T + Send + Sync + 'static,
{
        let mut state = init();
        let process = &process;
        consume_inner_loop::<M, T, P>(
            &mut state,
            process,
        );
}

fn consume_one<'a, 's, M, T, P>(state: &'s mut T, process: &P) -> bool
    where M: for<'x> capnp::traits::Owned<'x>,
          P: Fn(&mut T, Vec<<M as capnp::traits::Owned>::Reader>) -> bool,
 {
    let readers: Vec<M::Reader> = unsafe { std::mem::uninitialized() };
    process(state, readers)
}

fn consume_inner_loop<M, T, Process>(
    state: &mut T,
    process: &Process,
) -> Result<bool, ()>
where
    M: for<'x> capnp::traits::Owned<'x>,
    Process: Fn(&mut T, Vec<<M as capnp::traits::Owned>::Reader>) -> bool,
    T: 'static,
{
    let do_stop = consume_one::<M, T, Process>(state, process);

    Ok(do_stop)
}

pub trait BatchConsumerService: ConsumerService {
    fn call_batch(&mut self, _item: Vec<<Self::Item as capnp::traits::Owned>::Reader>);
}

pub trait ConsumerService {
    type Item: for<'x> capnp::traits::Owned<'x>;

    fn call(&mut self, _item: <Self::Item as capnp::traits::Owned>::Reader) {
        unimplemented!()
    }

    /// return true when you want to stop
    fn stop(&self) -> bool {
        false
    }
}

pub trait NewConsumerService {
    type Instance: ConsumerService<Item = Self::Item>;
    type Item: for<'x> capnp::traits::Owned<'x>;

    fn new_service(&self) -> Self::Instance;
}

impl<F, R> NewConsumerService for F
where
    F: Fn() -> R,
    R: ConsumerService,
{
    type Instance = R;
    type Item = R::Item;

    fn new_service(&self) -> Self::Instance {
        self()
    }
}

pub struct Consumer {}

impl Consumer {
    pub fn new() -> Self {
        Consumer {}
    }

    pub fn batch<N>(self, new: N) where
          N: NewConsumerService + Send + Sync + 'static,
          N::Item: for<'x> capnp::traits::Owned<'x>,
          N::Instance: BatchConsumerService,
    {
        consume_loop::<_, N::Item, _, _>(
            move || {
                new.new_service()
            },
            move |service, batch: Vec<_>| {
                service.call_batch(batch);
                service.stop()
            },
        )
    }
}