#[macro_use]
extern crate cpython;
extern crate futures;
extern crate librespot;
extern crate tokio_core;

use cpython::{PyResult, PyObject, Python, PythonObject};
use futures::Future;
use std::thread;
use tokio_core::reactor::Core;
use std::cell::RefCell;

// Workaround rust-lang/rust#28796
trait Callback : Send {
    fn call(self: Box<Self>, py: Python) -> PyResult<PyObject>;
}
impl <F: Send + for<'a> FnOnce(Python<'a>) -> PyResult<PyObject>> Callback for F {
    fn call(self: Box<Self>, py: Python) -> PyResult<PyObject> {
        (*self)(py)
    }
}

py_class!(class PyFuture |py| {
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

py_class!(class Session |py| {
    data session : librespot::session::Session;

    @classmethod def connect(_cls, username: String, password: String) -> PyResult<PyFuture> {
        use librespot::session::Config;
        use librespot::authentication::Credentials;

        let config = Config::default();
        let credentials = Credentials::with_password(username, password);

        let (tx, rx) = futures::sync::oneshot::channel();
        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let handle = core.handle();

            let session = core.run(librespot::session::Session::connect(config, credentials, None, handle)).unwrap();

            let _ = tx.send(session);

            core.run(futures::future::empty::<(), ()>()).unwrap();
        });

        PyFuture::new(py, rx, |py, result| {
            let session = result.unwrap();
            Session::create_instance(py, session)
        })
    }

    def player(&self) -> PyResult<Player> {
        let backend = librespot::audio_backend::find(None).unwrap();
        let session = self.session(py).clone();

        let player = librespot::player::Player::new(session, None, move || (backend)(None));
        Player::create_instance(py, player)
    }
});

py_class!(class Player |py| {
    data player : librespot::player::Player;

    def load(&self, track: SpotifyId, play: bool = true, position_ms: u32 = 0) -> PyResult<PyFuture> {
        let player = self.player(py);
        let track = *track.id(py);

        let end_of_track = player.load(track, play, position_ms);
        PyFuture::new(py, end_of_track, |py, _result| {
            Ok(py.None())
        })
    }
});

py_class!(class SpotifyId |py| {
    data id : librespot::util::SpotifyId;

    def __new__(_cls, base62: &str) -> PyResult<SpotifyId> {
        let id = librespot::util::SpotifyId::from_base62(base62);
        SpotifyId::create_instance(py, id)
    }
});

py_module_initializer!(librespot, initlibrespot, PyInit_librespot, |py, m| {
    m.add_class::<Session>(py)?;
    m.add_class::<SpotifyId>(py)?;
    Ok(())
});
