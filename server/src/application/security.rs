use actix_web::dev::{Transform, Service, ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage, HttpResponse};
use std::future::{ready, Ready, Future};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Security;

impl Security {
    pub fn new() -> Security {
        Security
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
        ready(Ok(SecurityMiddleware { service }))
    }
}

pub struct SecurityMiddleware<S> {
    service: S,
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
        let authCookie = req.cookie("X-Authorization-Token");

        match authCookie {
            Some(cookie) => {
                println!("Hi from Security Middleware. You requested: {}; auth-cookie: {}", req.path(), cookie);
                let fut = self.service.call(req);

                Box::pin(async move {
                    let res = fut.await?;
                    println!("Hi from response");
                    Ok(res)
                })
            },
            None => {
                Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Not found authorization token"))})
            }
        }
    }
}