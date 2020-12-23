use std::future::{Future, Ready, ready};
use std::pin::Pin;
use std::task::{Poll, Context};

use actix_web::{Error, HttpMessage};
use actix_web::dev::{ServiceRequest, ServiceResponse, Service, Transform};

use jsonwebtoken::{Validation, Algorithm};

use crate::security::SecurityContext;

pub struct AuthorizationMiddleware<S> {
    service: S,
    only_developer: bool
}

impl<S,B> Service for AuthorizationMiddleware<S>
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
        let authorized =
            {
                let extensions = &req.extensions();
                let context= extensions.get::<SecurityContext>();
                match context {
                    Some(ctx) => {
                        if self.only_developer {
                            ctx.groups.contains("DEVELOPER")
                        } else {
                            ctx.groups.contains("BASE_ACCESS")
                        }
                    },
                    None => false
                }
            };

        if authorized {
            let fut = self.service.call(req);

            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            Box::pin(async { Err(actix_web::error::ErrorUnauthorized("You are not authenticated"))})
        }
    }

}

#[derive(Clone)]
pub struct Authorized {
    only_developer: bool
}

impl Authorized {
    pub fn all() -> Self {
        Self { only_developer: false}
    }
    pub fn developers() -> Self {
        Self { only_developer: true}
    }
}

impl <S,B> Transform<S> for Authorized
    where
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthorizationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorizationMiddleware { service, only_developer: self.only_developer }))
    }
}