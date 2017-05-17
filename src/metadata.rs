use cpython::{PyResult, Python, PythonObject};
use futures;
use librespot;
use pyfuture::PyFuture;
use tokio_core::reactor::Remote;
use SpotifyId;

py_class!(pub class Track |py| {
    data session : librespot::session::Session;
    data handle : Remote;
    data track : librespot::metadata::Track;

    def id(&self) -> PyResult<SpotifyId> {
        SpotifyId::new(py, self.track(py).id)
    }

    def name(&self) -> PyResult<String> {
        Ok(self.track(py).name.clone())
    }

    def album(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let album = self.track(py).album;

        Album::get(py, session, handle, album)
    }

    def artists(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let artists = self.track(py).artists.clone();
        Artist::get_all(py, session, handle, artists)
    }
});

py_class!(pub class Album |py| {
    data session : librespot::session::Session;
    data handle : Remote;
    data album : librespot::metadata::Album;

    def id(&self) -> PyResult<SpotifyId> {
        SpotifyId::new(py, self.album(py).id)
    }

    def name(&self) -> PyResult<String> {
        Ok(self.album(py).name.clone())
    }

    def artists(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let artists = self.album(py).artists.clone();
        Artist::get_all(py, session, handle, artists)
    }

    def tracks(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let artists = self.album(py).tracks.clone();
        Track::get_all(py, session, handle, artists)
    }
});

py_class!(pub class Artist |py| {
    data _session : librespot::session::Session;
    data _handle : Remote;
    data artist : librespot::metadata::Artist;

    def id(&self) -> PyResult<SpotifyId> {
        SpotifyId::new(py, self.artist(py).id)
    }

    def name(&self) -> PyResult<String> {
        Ok(self.artist(py).name.clone())
    }
});

fn get<T, F, O>(py: Python,
                session: librespot::session::Session,
                handle : Remote,
                id: librespot::util::SpotifyId,
                create_instance: F) -> PyResult<PyFuture>
    where
        T: librespot::metadata::MetadataTrait + Send,
        F: FnOnce(Python, librespot::session::Session, Remote, T) -> PyResult<O> + Send + 'static,
        O: PythonObject
{
    let future = session.metadata().get::<T>(id);
    PyFuture::new(py, handle.clone(), future, |py, result| {
        create_instance(py, session, handle, result.unwrap())
    })
}

fn get_all<T, F, O, I>(py: Python,
                       session: librespot::session::Session,
                       handle : Remote,
                       ids: I,
                       create_instance: F) -> PyResult<PyFuture>
    where
        T: librespot::metadata::MetadataTrait + Send,
        F: Fn(Python, librespot::session::Session, Remote, T) -> PyResult<O> + Send + 'static,
        O: PythonObject,
        I: IntoIterator<Item = librespot::util::SpotifyId>,
        I::IntoIter: 'static
{
    let session_ = session.clone();
    let futures = ids.into_iter().map(move |id| {
        session_.metadata().get::<T>(id)
    });

    let future = futures::future::join_all(futures);

    PyFuture::new(py, handle.clone(), future, move |py, result| {
        let objects = result.unwrap();
        let objects = objects.into_iter().map(|artist| {
            create_instance(py, session.clone(), handle.clone(), artist)
        }).collect::<PyResult<Vec<_>>>()?;

        Ok(objects)
    })
}

impl Track {
    pub fn get(py: Python,
               session: librespot::session::Session,
               handle : Remote,
               id: librespot::util::SpotifyId) -> PyResult<PyFuture>
    {
        get(py, session, handle, id, Track::create_instance)
    }

    pub fn get_all<I>(py: Python,
                      session: librespot::session::Session,
                      handle : Remote,
                      ids: I) -> PyResult<PyFuture>
        where I: IntoIterator<Item = librespot::util::SpotifyId>,
              I::IntoIter: 'static
    {
        get_all(py, session, handle, ids, Track::create_instance)
    }
}

impl Album {
    pub fn get(py: Python,
               session: librespot::session::Session,
               handle : Remote,
               id: librespot::util::SpotifyId) -> PyResult<PyFuture>
    {
        get(py, session, handle, id, Album::create_instance)
    }

    pub fn get_all<I>(py: Python,
                      session: librespot::session::Session,
                      handle : Remote,
                      ids: I) -> PyResult<PyFuture>
        where I: IntoIterator<Item = librespot::util::SpotifyId>,
              I::IntoIter: 'static
    {
        get_all(py, session, handle, ids, Album::create_instance)
    }
}

impl Artist {
    pub fn get(py: Python,
               session: librespot::session::Session,
               handle : Remote,
               id: librespot::util::SpotifyId) -> PyResult<PyFuture>
    {
        get(py, session, handle, id, Artist::create_instance)
    }

    pub fn get_all<I>(py: Python,
                      session: librespot::session::Session,
                      handle : Remote,
                      ids: I) -> PyResult<PyFuture>
        where I: IntoIterator<Item = librespot::util::SpotifyId>,
              I::IntoIter: 'static
    {
        get_all(py, session, handle, ids, Artist::create_instance)
    }
}
