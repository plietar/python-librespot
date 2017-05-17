use cpython::{PyResult, PyObject, Python, PythonObject, ToPyObject, ObjectProtocol, PyClone};
use futures::Future;
use std::mem::replace;
use std::sync::{Arc, Mutex, Condvar};
use tokio_core::reactor::Remote;

pub struct FutureShared {
    data: Option<PyResult<PyObject>>,
    callbacks: Vec<PyObject>,
}

pub struct FutureInner {
    shared: Mutex<FutureShared>,
    cond: Condvar,
}

py_class!(pub class PyFuture |py| {
    data inner : Arc<FutureInner>;

    def poll(&self) -> PyResult<Option<PyObject>> {
        let inner = self.inner(py);
        let shared = inner.shared.lock().unwrap();
        match shared.data {
            Some(Ok(ref val)) => Ok(Some(val.clone_ref(py))),
            Some(Err(ref err)) => Err(err.clone_ref(py)),
            None => Ok(None),
        }
    }

    def wait(&self) -> PyResult<PyObject> {
        let inner = self.inner(py).clone();

        py.allow_threads(|| {
            let mut shared = inner.shared.lock().unwrap();

            loop {
                if let Some(ref result) = shared.data {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    return result.as_ref()
                        .map(|o| o.clone_ref(py))
                        .map_err(|o| o.clone_ref(py));
                }

                shared = inner.cond.wait(shared).unwrap();
            }
        })
    }

    def add_callback(&self, cb: PyObject) -> PyResult<PyObject> {
        let inner = self.inner(py);
        let mut shared = inner.shared.lock().unwrap();
        let shared = &mut *shared;
        match shared.data {
            Some(Ok(ref val)) => {
                let val = val.clone_ref(py);
                cb.call(py, (val,) , None)?;
                Ok(py.None())
            }

            // TODO: notify cb on error
            Some(Err(_)) => Ok(py.None()),
            None => {
                shared.callbacks.push(cb);
                Ok(py.None())
            }
        }
    }
});

impl PyFuture {
    pub fn new<F, T, U>(py: Python, handle: Remote, future: F, then: T) -> PyResult<PyFuture>
        where F: Future + Send + 'static,
              T: FnOnce(Python, Result<F::Item, F::Error>) -> PyResult<U> + Send + 'static,
              U: ToPyObject
    {
        let shared = FutureShared {
            data: None,
            callbacks: Vec::new(),
        };
        let inner = Arc::new(FutureInner {
            shared: Mutex::new(shared),
            cond: Condvar::new(),
        });
        let pyfuture = PyFuture::create_instance(py, inner.clone());

        handle.spawn(move |_| {
            future.then(move |result| {
                let mut shared = inner.shared.lock().unwrap();
                let gil = Python::acquire_gil();
                let py = gil.python();
                let value = then(py, result).map(|o| o.into_py_object(py).into_object());

                shared.data = match value {
                    Ok(ref val) => Some(Ok(val.clone_ref(py))),
                    Err(ref err) => Some(Err(err.clone_ref(py))),
                };

                let callbacks = replace(&mut shared.callbacks, Vec::new());
                drop(shared);

                // TODO: notify cb on error
                if let Ok(ref value) = value {
                    for cb in callbacks {
                        let _ = cb.call(py, (value,), None);
                    }
                }

                inner.cond.notify_all();

                Ok(())
            })
        });

        pyfuture
    }
}
