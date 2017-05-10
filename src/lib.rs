#[macro_use]
extern crate cpython;
extern crate futures;
extern crate librespot;
extern crate tokio_core;

use cpython::PyResult;

mod pyfuture;
mod player;
mod session;

py_class!(pub class SpotifyId |py| {
    data id : librespot::util::SpotifyId;

    def __new__(_cls, base62: &str) -> PyResult<SpotifyId> {
        let id = librespot::util::SpotifyId::from_base62(base62);
        SpotifyId::create_instance(py, id)
    }
});

py_module_initializer!(librespot, initlibrespot, PyInit_librespot, |py, m| {
    m.add_class::<session::Session>(py)?;
    m.add_class::<SpotifyId>(py)?;
    Ok(())
});
