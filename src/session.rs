use librespot;
use std::thread;
use tokio_core::reactor::Core;
use cpython::PyResult;
use futures;

use pyfuture::PyFuture;
use player::Player;

py_class!(pub class Session |py| {
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
        let session = self.session(py).clone();

        Player::new(py, session)
    }
});
