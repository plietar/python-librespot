use librespot;
use cpython::{PyResult, Python, PythonObject};
use pyfuture::PyFuture;
use futures;
use SpotifyId;

py_class!(pub class Track |py| {
    data session : librespot::session::Session;
    data track : librespot::metadata::Track;

    def id(&self) -> PyResult<SpotifyId> {
        SpotifyId::new(py, self.track(py).id)
    }

    def name(&self) -> PyResult<String> {
        Ok(self.track(py).name.clone())
    }

    def album(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let album = self.track(py).album;

        Album::get(py, session, album)
    }

    def artists(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let artists = self.track(py).artists.clone();
        Artist::get_all(py, session, artists)
    }
});

py_class!(pub class Album |py| {
    data session : librespot::session::Session;
    data album : librespot::metadata::Album;

    def id(&self) -> PyResult<SpotifyId> {
        SpotifyId::new(py, self.album(py).id)
    }

    def name(&self) -> PyResult<String> {
        Ok(self.album(py).name.clone())
    }

    def artists(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let artists = self.album(py).artists.clone();
        Artist::get_all(py, session, artists)
    }

    def tracks(&self) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let artists = self.album(py).tracks.clone();
        Track::get_all(py, session, artists)
    }
});

py_class!(pub class Artist |py| {
    data _session : librespot::session::Session;
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
                id: librespot::util::SpotifyId,
                create_instance: F) -> PyResult<PyFuture>
    where
        T: librespot::metadata::MetadataTrait + Send,
        F: FnOnce(Python, librespot::session::Session, T) -> PyResult<O> + Send + 'static,
        O: PythonObject
{
    let future = session.metadata().get::<T>(id);
    PyFuture::new(py, future, |py, result| {
        create_instance(py, session, result.unwrap())
    })
}

fn get_all<T, F, O, I>(py: Python,
                       session: librespot::session::Session,
                       ids: I,
                       create_instance: F) -> PyResult<PyFuture>
    where
        T: librespot::metadata::MetadataTrait + Send,
        F: Fn(Python, librespot::session::Session, T) -> PyResult<O> + Send + 'static,
        O: PythonObject,
        I: IntoIterator<Item = librespot::util::SpotifyId>,
        I::IntoIter: 'static
{
    let session_ = session.clone();
    let futures = ids.into_iter().map(move |id| {
        session_.metadata().get::<T>(id)
    });

    let future = futures::future::join_all(futures);

    PyFuture::new(py, future, move |py, result| {
        let objects = result.unwrap();
        let objects = objects.into_iter().map(|artist| {
            create_instance(py, session.clone(), artist)
        }).collect::<PyResult<Vec<_>>>()?;

        Ok(objects)
    })
}

impl Track {
    pub fn get(py: Python,
               session: librespot::session::Session,
               id: librespot::util::SpotifyId) -> PyResult<PyFuture>
    {
        get(py, session, id, Track::create_instance)
    }

    pub fn get_all<I>(py: Python,
                      session: librespot::session::Session,
                      ids: I) -> PyResult<PyFuture>
        where I: IntoIterator<Item = librespot::util::SpotifyId>,
              I::IntoIter: 'static
    {
        get_all(py, session, ids, Track::create_instance)
    }
}

impl Album {
    pub fn get(py: Python,
               session: librespot::session::Session,
               id: librespot::util::SpotifyId) -> PyResult<PyFuture>
    {
        get(py, session, id, Album::create_instance)
    }

    pub fn get_all<I>(py: Python,
                      session: librespot::session::Session,
                      ids: I) -> PyResult<PyFuture>
        where I: IntoIterator<Item = librespot::util::SpotifyId>,
              I::IntoIter: 'static
    {
        get_all(py, session, ids, Album::create_instance)
    }
}

impl Artist {
    pub fn get(py: Python,
               session: librespot::session::Session,
               id: librespot::util::SpotifyId) -> PyResult<PyFuture>
    {
        get(py, session, id, Artist::create_instance)
    }

    pub fn get_all<I>(py: Python,
                      session: librespot::session::Session,
                      ids: I) -> PyResult<PyFuture>
        where I: IntoIterator<Item = librespot::util::SpotifyId>,
              I::IntoIter: 'static
    {
        get_all(py, session, ids, Artist::create_instance)
    }
}
