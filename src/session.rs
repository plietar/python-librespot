use librespot;
use std::thread;
use tokio_core::reactor::Core;
use cpython::PyResult;
use futures;

use pyfuture::PyFuture;
use player::Player;
use metadata::{Track, Album, Artist};
use webtoken::Token;
use SpotifyId;

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

    def get_track(&self, track: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let track = *track.id(py);

        Track::get(py, session, track)
    }

    def get_album(&self, album: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let album = *album.id(py);

        Album::get(py, session, album)
    }

    def get_artist(&self, artist: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let artist = *artist.id(py);

        Artist::get(py, session, artist)
    }

    def web_token(&self, client_id: &str, scopes: &str) -> PyResult<PyFuture> {
        let session = self.session(py);
        Token::get(py, session, client_id, scopes)
    }
});
