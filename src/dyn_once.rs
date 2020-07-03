use std::mem;

pub(crate) struct DynOnce<'a, T, U> {
    inner: DynInner<T, U>,
    _marker: std::marker::PhantomData<&'a mut ()>,
}

enum DynInner<T, U> {
    Visitor(T),
    Value(U),
    Empty,
}

impl<'a, T, U> DynOnce<'a, T, U> {
    pub unsafe fn new(
        visitor: T,
        _: &'a InvariantLifetime<'a>,
    ) -> (DynOnce<'a, T, U>, Visitor<'a>) {
        (
            DynOnce {
                inner: DynInner::Visitor(visitor),
                _marker: std::marker::PhantomData,
            },
            Visitor(InvariantLifetime::default()),
        )
    }

    pub fn as_visitor<'b>(&'b self, _token: &'b Visitor<'a>) -> &'b T {
        match &self.inner {
            DynInner::Visitor(visitor) => visitor,
            _ => unreachable!(),
        }
    }

    pub fn take_visitor(&mut self, _token: Visitor<'a>) -> T {
        match mem::replace(&mut self.inner, DynInner::Empty) {
            DynInner::Visitor(visitor) => visitor,
            // SAFETY The token only exists if `Visitor` is set
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    pub fn take_value(&mut self, _token: Value<'a>) -> U {
        match mem::replace(&mut self.inner, DynInner::Empty) {
            DynInner::Value(value) => value,
            // SAFETY The token only exists if `Value` is set
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    pub fn set_value(&mut self, value: U) -> Value<'a> {
        self.inner = DynInner::Value(value);
        Value(InvariantLifetime::default())
    }
}

pub struct Value<'a>(InvariantLifetime<'a>);
pub struct Visitor<'a>(InvariantLifetime<'a>);

#[derive(Default)]
pub struct InvariantLifetime<'a>(std::marker::PhantomData<fn(&'a ()) -> &'a ()>);

// A macro is cheaper than a function
#[macro_export]
#[doc(hidden)]
macro_rules! dyn_once {
    ($visitor: ident, $token: ident) => {
        // Copied from the compact_arena crate
        let tag = $crate::dyn_once::InvariantLifetime::default();
        let (mut once, $token) = unsafe { $crate::dyn_once::DynOnce::new($visitor, &tag) };
        let $visitor = &mut once;

        let _guard;
        // this doesn't make it to MIR, but ensures that borrowck will not
        // unify the lifetimes of two macro calls by binding the lifetime to
        // drop scope
        if false {
            struct Guard<'tag>(&'tag $crate::dyn_once::InvariantLifetime<'tag>);
            impl<'tag> ::core::ops::Drop for Guard<'tag> {
                fn drop(&mut self) {}
            }
            _guard = Guard(&tag);
        }
    };
}
