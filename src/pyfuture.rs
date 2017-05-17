use cpython::{PyResult, PyObject, Python, PythonObject, ToPyObject, ObjectProtocol};
use futures::executor;
use futures::{Future, Async};
use std::cell::RefCell;
use std::sync::Arc;
use tokio_core::reactor::Remote;

pub trait Callback : Send {
    fn poll(&mut self, py: Python) -> PyResult<Option<PyObject>>;
    fn wait(&mut self, py: Python) -> PyResult<PyObject>;
    fn spawn(&mut self, py: Python, cb: PyObject) -> PyResult<PyObject>;
}

struct FutureData<F, T> {
    handle: Remote,
    future: Option<executor::Spawn<F>>,
    then: Option<T>,
}

struct NoopUnpark;
impl executor::Unpark for NoopUnpark {
    fn unpark(&self) {}
}

impl <F, T, U> Callback for FutureData<F, T>
    where F: Future + Send + 'static,
          T: FnOnce(Python, Result<F::Item, F::Error>) -> PyResult<U> + Send + 'static,
          U: ToPyObject
{
    fn poll(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        let result = {
            let future = self.future.as_mut().expect("Future already consumed");
            match future.poll_future(Arc::new(NoopUnpark)) {
                Ok(Async::Ready(v)) => Ok(v),
                Err(e) => Err(e),
                Ok(Async::NotReady) => return Ok(None),
            }
        };

        self.future = None;
        let then = self.then.take().unwrap();
        then(py, result).map(|o| Some(o.into_py_object(py).into_object()))
    }

    fn wait(&mut self, py: Python) -> PyResult<PyObject> {
        let mut future = self.future.take().expect("Future already consumed");
        let result = py.allow_threads(|| future.wait_future());

        let then = self.then.take().unwrap();
        then(py, result).map(|o| o.into_py_object(py).into_object())
    }

    fn spawn(&mut self, py: Python, cb: PyObject) -> PyResult<PyObject> {
        let future = self.future.take().expect("Future already consumed").into_inner();
        let then = self.then.take().unwrap();

        self.handle.spawn(move |_handle| {
            future.then(move |result| {
                let gil = Python::acquire_gil();
                let py = gil.python();

                let arg = then(py, result).unwrap().into_py_object(py).into_object();
                cb.call(py, (arg,) , None).unwrap();

                Ok(())
            })
        });

        Ok(py.None())
    }
}

py_class!(pub class PyFuture |py| {
    data callback : RefCell<Box<Callback>>;

    def poll(&self) -> PyResult<Option<PyObject>> {
        self.callback(py).borrow_mut().poll(py)
    }

    def wait(&self) -> PyResult<PyObject> {
        self.callback(py).borrow_mut().wait(py)
    }

    def spawn(&self, cb: PyObject) -> PyResult<PyObject> {
        self.callback(py).borrow_mut().spawn(py, cb)
    }
});

impl PyFuture {
    pub fn new<F, T, U>(py: Python, handle: Remote, future: F, then: T) -> PyResult<PyFuture>
        where F: Future + Send + 'static,
              T: FnOnce(Python, Result<F::Item, F::Error>) -> PyResult<U> + Send + 'static,
              U: ToPyObject
    {
        PyFuture::create_instance(py, RefCell::new(Box::new(FutureData {
            handle: handle,
            future: Some(executor::spawn(future)),
            then: Some(then),
        })))
    }
}

