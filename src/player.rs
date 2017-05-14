use librespot;
use SpotifyId;
use cpython::{PyResult, PyObject, Python};
use pyfuture::PyFuture;

py_class!(pub class Player |py| {
    data player : librespot::player::Player;

    def load(&self, track: SpotifyId, play: bool = true, position_ms: u32 = 0) -> PyResult<PyFuture> {
        let player = self.player(py);
        let track = *track.id(py);

        let end_of_track = player.load(track, play, position_ms);
        PyFuture::new(py, end_of_track, |_py, _result| {
            Ok(true)
        })
    }

    def play(&self) -> PyResult<PyObject> {
        let player = self.player(py);
        player.play();
        Ok(py.None())
    }

    def pause(&self) -> PyResult<PyObject> {
        let player = self.player(py);
        player.pause();
        Ok(py.None())
    }
});

impl Player {
    pub fn new(py: Python, session: librespot::session::Session) -> PyResult<Player> {
        let backend = librespot::audio_backend::find(None).unwrap();
        let player = librespot::player::Player::new(session, None, move || (backend)(None));
        Player::create_instance(py, player)
    }
}

