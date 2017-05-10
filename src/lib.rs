#[macro_use]
extern crate cpython;
extern crate tokio_core;
extern crate librespot;

use std::cell::RefCell;
use cpython::{PyResult, PyObject};
use tokio_core::reactor::Core;

thread_local! {
    pub static CORE: RefCell<Core> = RefCell::new(Core::new().unwrap());
}

py_class!(class Session |py| {
    data session : librespot::session::Session;

    def __new__(_cls, username: String, password: String) -> PyResult<Session> {
        use librespot::session::Config;
        use librespot::authentication::Credentials;

        CORE.with(|core| {
            let mut core = core.borrow_mut();
            let handle = core.handle();

            let config = Config::default();
            let credentials = Credentials::with_password(username, password);
            let session = core.run(librespot::session::Session::connect(config, credentials, None, handle)).unwrap();

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

    def play(&self, track: SpotifyId) -> PyResult<PyObject> {
        CORE.with(|core| {
            let mut core = core.borrow_mut();
            let player = self.player(py);
            let track = *track.id(py);

            core.run(player.load(track, true, 0)).unwrap();

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
