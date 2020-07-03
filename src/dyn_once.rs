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
    pub fn new(visitor: T) -> (DynOnce<'a, T, U>, Visitor<'a>) {
        (
            DynOnce {
                inner: DynInner::Visitor(visitor),
                _marker: std::marker::PhantomData,
            },
            Visitor(std::marker::PhantomData),
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
        Value(std::marker::PhantomData)
    }
}

pub struct Value<'a>(std::marker::PhantomData<&'a mut ()>);
pub struct Visitor<'a>(std::marker::PhantomData<&'a mut ()>);

// A macro is cheaper than a function
#[macro_export]
#[doc(hidden)]
macro_rules! dyn_once {
    ($visitor: expr, |$visitor_ident: ident, $token: ident| $expr: expr) => {{
        let (mut once, $token) = DynOnce::new($visitor);
        let $visitor_ident = &mut once;
        $expr
    }};
}
