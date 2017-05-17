use librespot;
use std::thread;
use tokio_core::reactor::Core;
use cpython::PyResult;
use futures;
use tokio_core::reactor::Remote;
use futures::Future;

use pyfuture::PyFuture;
use player::Player;
use metadata::{Track, Album, Artist};
use webtoken::Token;
use SpotifyId;

py_class!(pub class Session |py| {
    data session : librespot::session::Session;
    data handle: Remote;

    @classmethod def connect(_cls, username: String, password: String) -> PyResult<PyFuture> {
        use librespot::session::Config;
        use librespot::authentication::Credentials;

        let config = Config::default();
        let credentials = Credentials::with_password(username, password);

        let (session_tx, session_rx) = futures::sync::oneshot::channel();
        let (handle_tx, handle_rx) = futures::sync::oneshot::channel();

        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let handle = core.handle();

            let _ = handle_tx.send(handle.remote().clone());

            let session = core.run(librespot::session::Session::connect(config, credentials, None, handle)).unwrap();

            let _ = session_tx.send(session);

            core.run(futures::future::empty::<(), ()>()).unwrap();
        });

        let handle = handle_rx.wait().unwrap();

        PyFuture::new(py, handle.clone(), session_rx, move |py, result| {
            let session = result.unwrap();
            Session::create_instance(py, session, handle)
        })
    }

    def player(&self) -> PyResult<Player> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();

        Player::new(py, session, handle)
    }

    def get_track(&self, track: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let track = *track.id(py);

        Track::get(py, session, handle, track)
    }

    def get_album(&self, album: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let album = *album.id(py);

        Album::get(py, session, handle, album)
    }

    def get_artist(&self, artist: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let artist = *artist.id(py);

        Artist::get(py, session, handle, artist)
    }

    def web_token(&self, client_id: &str, scopes: &str) -> PyResult<PyFuture> {
        let session = self.session(py);
        let handle = self.handle(py).clone();
        Token::get(py, session, handle, client_id, scopes)
    }
});
