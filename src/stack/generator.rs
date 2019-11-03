use crate::{
    stack::engine::{advance, Airlock, Co},
    GeneratorState,
};
use std::{cell::UnsafeCell, future::Future, mem, pin::Pin, ptr};

pub struct Gen<Y, F: Future> {
    state: State<Y, F>,
}

pub struct State<Y, F: Future> {
    airlock: Airlock<Y>,
    future: F,
}

impl<Y, F: Future> Gen<Y, F> {
    pub unsafe fn __macro_internal_popuate<'y>(
        this: &mut mem::MaybeUninit<Self>,
        start: impl FnOnce(Co<'y, Y>) -> F,
    ) where
        Y: 'y,
    {
        let p = &mut (*this.as_mut_ptr()).state as *mut State<Y, F>;

        let airlock = UnsafeCell::new(None);
        ptr::write(&mut (*p).airlock, airlock);

        let future = start(Co {
            airlock: &(*p).airlock,
        });
        ptr::write(&mut (*p).future, future);
    }

    pub fn __macro_internal_resume(
        self: Pin<&mut Self>,
    ) -> GeneratorState<Y, F::Output> {
        let state: *mut State<Y, F> = &self.state as *const _ as *mut _;
        unsafe {
            let airlock = &mut (*state).airlock;
            let future = self.map_unchecked_mut(|s| &mut s.state.future);
            advance(future, airlock)
        }
    }
}

impl<Y, F: Future> Drop for Gen<Y, F> {
    fn drop(&mut self) {
        let state: *mut State<Y, F> = &self.state as *const _ as *mut _;
        unsafe {
            ptr::drop_in_place(&mut (*state).future);
            ptr::drop_in_place(&mut (*state).airlock);
        }
    }
}