use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use futures_util::future::BoxFuture;
use crate::core::object::Object;

pub(crate) type PinFutureObj<Output> = Pin<Box<dyn Future<Output = Output>>>;

pub trait ModelCallback {
    fn call(&self, object: &Object) -> BoxFuture<'static, ()>;
}

impl<T, F> ModelCallback for Arc<T> where T: Fn(&Object) -> F, F: Future<Output = ()> + 'static + Send {
    fn call(&self, obj: &Object) -> BoxFuture<'static, ()> {
        Box::pin(self(obj))
    }
}
