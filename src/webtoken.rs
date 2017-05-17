use librespot;
use cpython::{PyResult, Python};
use pyfuture::PyFuture;
use tokio_core::reactor::Remote;

py_class!(pub class Token |py| {
    data token : librespot::keymaster::Token;

    def access_token(&self) -> PyResult<String> {
        Ok(self.token(py).access_token.clone())
    }

    def token_type(&self) -> PyResult<String> {
        Ok(self.token(py).token_type.clone())
    }

    def expires_in(&self) -> PyResult<u32> {
        Ok(self.token(py).expires_in)
    }

    def scope(&self) -> PyResult<Vec<String>> {
        Ok(self.token(py).scope.clone())
    }
});

impl Token {
    pub fn get(py: Python,
               session: &librespot::session::Session,
               handle : Remote,
               client_id: &str, scopes: &str)
        -> PyResult<PyFuture>
    {
        let future = librespot::keymaster::get_token(session, client_id, scopes);
        PyFuture::new(py, handle, future, move |py, result| {
            let token = result.unwrap();
            Token::create_instance(py, token)
        })
    }
}
