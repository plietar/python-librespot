use futures::Future;
use std::cell::RefCell;
use cpython::{PyResult, PyObject, Python, PythonObject};

// Workaround rust-lang/rust#28796
pub trait Callback : Send {
    fn call(self: Box<Self>, py: Python) -> PyResult<PyObject>;
}
impl <F: Send + for<'a> FnOnce(Python<'a>) -> PyResult<PyObject>> Callback for F {
    fn call(self: Box<Self>, py: Python) -> PyResult<PyObject> {
        (*self)(py)
    }
}

py_class!(pub class PyFuture |py| {
    data callback : RefCell<Option<Box<Callback>>>;

    def wait(&self) -> PyResult<PyObject> {
        let callback = self.callback(py).borrow_mut().take().expect("Future already completed");
        callback.call(py)
    }
});

impl PyFuture {
    pub fn new<F, T, U>(py: Python, future: F, then: T) -> PyResult<PyFuture>
        where F: Future + Send + 'static,
              T: FnOnce(Python, Result<F::Item, F::Error>) -> PyResult<U> + Send + 'static,
              U: PythonObject
    {
        PyFuture::create_instance(py, RefCell::new(Some(Box::new(move |py: Python| {
            let result = future.wait();
            then(py, result).map(PythonObject::into_object)
        }))))
    }
}

