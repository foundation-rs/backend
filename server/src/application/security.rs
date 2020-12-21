use actix_web::dev::{Transform, Service, ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage, HttpResponse};
use std::future::{ready, Ready, Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::sync::RwLock;

use lazy_static::lazy_static;

struct Inner {
    source: Box<Vec<u8>>,
    key:    jsonwebtoken::DecodingKey<'static>,
    token:  String,
}

impl Inner {
    pub fn new(token: String, key_file: PathBuf) -> Self {
        let mut file = File::open(key_file).map_err(|err| format!("Can not open config file: {}", err)).unwrap();
        let mut source = Vec::with_capacity(1024);
        file.read_to_end(&mut source).unwrap();

        let source = Box::new(source);

        let key = unsafe {
            let source: &'static Vec<u8> = std::mem::transmute(&*source);
            jsonwebtoken::DecodingKey::from_rsa_pem(source).unwrap()
        };

        Self { source, key, token}
    }
}

#[derive(Clone)]
pub struct Security {
    inner: Arc<Inner>,
}

impl Security {
    pub fn new(token_name: String, key_file: PathBuf) -> Security {
        let inner = Arc::new(Inner::new(token_name, key_file));
        Security { inner }
    }
}

impl <S,B> Transform<S> for Security
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityMiddleware { service, inner: self.inner.clone() }))
    }
}

pub struct SecurityMiddleware<S> {
    service: S,
    inner: Arc<Inner>,
}

impl<S,B> Service for SecurityMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let auth_cookie = req.cookie(&self.inner.token);

        match auth_cookie {
            Some(cookie) => {
                let cookie_value = cookie.value();
                let decode_result = jsonwebtoken::decode_header(cookie_value);

                match decode_result {
                    Ok(res) => {
                        println!("Hi from Security Middleware. You requested: {}", req.path());
                        let fut = self.service.call(req);

                        Box::pin(async move {
                            let res = fut.await?;
                            Ok(res)
                        })
                    },
                    Err(err) => {
                        Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Can not decode authorization token"))})
                    }
                }
            },
            None => {
                Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Not found authorization token"))})
            }
        }
    }
}