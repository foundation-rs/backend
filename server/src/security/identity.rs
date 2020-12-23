use std::collections::HashSet;
use std::fs::File;
use std::future::{Future, Ready, ready};
use std::io::Read;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Context};

use actix_web::{Error, HttpMessage};
use actix_web::dev::{ServiceRequest, ServiceResponse, Service, Transform};
use serde::{Serialize, Deserialize};

use jsonwebtoken::{Validation, Algorithm};

use crate::security::SecurityContext;

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

pub struct IdentityMiddleware<S> {
    service: S,
    inner: Arc<Inner>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp:    usize,     // Expiration time
    iat:    usize,     // Issued at
    iss:    String,    // Issuer
    sub:    String,    // Subject (user-id)
    groups: HashSet<String>, // Roles set
}

impl <S,B> IdentityMiddleware<S>
    where
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    fn construct_context(&mut self, req: &ServiceRequest) -> Result<(), String> {
        let auth_cookie = req.cookie(&self.inner.token);
        match auth_cookie {
            Some(cookie) => {
                let cookie_value = cookie.value();
                let decode_result = jsonwebtoken::decode::<Claims>(cookie_value, &self.inner.key, &Validation::new(Algorithm::RS256));
                match decode_result {
                    Ok(result) => {
                        let claims = result.claims;
                        req.extensions_mut().insert(SecurityContext::new(claims.sub, claims.groups));
                        Ok(())
                    },
                    Err(err) => {
                        Err(format!("Can not decode authorization token: {}", err))
                    }
                }
            },
            None => Ok(())
        }
    }

}

impl<S,B> Service for IdentityMiddleware<S>
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
        match self.construct_context(&req) {
            Ok(_) => {
                let fut = self.service.call(req);

                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            },
            Err(err) => {
                Box::pin(async { Err(actix_web::error::ErrorBadRequest(err))})
            }
        }
    }

}

#[derive(Clone)]
pub struct IdentityService {
    inner: Arc<Inner>,
}

impl IdentityService {
    pub fn new(token_name: String, key_file: PathBuf) -> Self {
        let inner = Arc::new(Inner::new(token_name, key_file));
        Self { inner }
    }
}

impl <S,B> Transform<S> for IdentityService
    where
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = IdentityMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IdentityMiddleware { service, inner: self.inner.clone() }))
    }
}