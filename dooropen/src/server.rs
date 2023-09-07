use crate::pin_handle::{init_handle, PinLevel, PinRegistry};
use async_trait::async_trait;
use dooropen_api::models::Status;
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use rppal::gpio::{Gpio, InputPin};
use std::marker::PhantomData;
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod};

use dooropen_api::models;

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    // let mut pin_int = PinInterrupter::new();

    let g = Gpio::new().expect("Gpio init failed!");
    let pin = g.get(3).expect("Couldn't get gpio pin 3").into_input();
    // pin_int.register_pin(&mut pin);

    // let jh = pin_int.start();
    // TODO better thread handle
    let (_pin_handle, mut pin_reg) = init_handle::<InputPin>();
    pin_reg.register_pin(pin);

    let server = Server::new(pin_reg);

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service =
        dooropen_api::server::context::MakeAddContext::<_, EmptyContext>::new(service);

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())
                .expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM)
                .expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem")
                .expect("Failed to set certificate chain");
            ssl.check_private_key()
                .expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let tcp_listener = TcpListener::bind(&addr).await.unwrap();

            loop {
                if let Ok((tcp, _)) = tcp_listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        Http::new()
                            .serve_connection(tls, service)
                            .await
                            .map_err(|_| ())
                    });
                }
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr)
            .serve(service)
            .await
            .unwrap()
    }
}

#[derive(Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
    pin_reg: PinRegistry<InputPin>,
}

impl<C> Server<C> {
    pub fn new(pin_dir: PinRegistry<InputPin>) -> Self {
        Server {
            marker: PhantomData,
            pin_reg: pin_dir,
        }
    }
}

use dooropen_api::server::MakeService;
use dooropen_api::{Api, DoorStatusResponse, PingResponse};
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
    /// Get status of the door
    async fn door_status(&self, context: &C) -> Result<DoorStatusResponse, ApiError> {
        let context = context.clone();
        info!("door_status() - X-Span-ID: {:?}", context.get().0.clone());
        let state = match self.pin_reg.get_pin_level(3) {
            Some(state) => match state {
                PinLevel::High => true,
                PinLevel::Low => false,
            },
            None => {
                println!("Couldn't get state of 3, no entry in dictionary !");
                return Err(ApiError(
                    "Couldn't get the state, no entry in dictionary".into(),
                ));
            }
        };
        println!("New DoorStatus Request!");
        Ok(DoorStatusResponse::Success(models::DoorStatus {
            header: None,
            lock_status: Some(state),
        }))
    }

    /// Ping the REST API
    async fn ping(&self, context: &C) -> Result<PingResponse, ApiError> {
        let context = context.clone();
        println!("pinged");
        info!("ping() - X-Span-ID: {:?}", context.get().0.clone());
        //Err(ApiError("Generic failure".into()))
        Ok(PingResponse::Success(Status {
            message: "all ok".to_string(),
        }))
    }
}
