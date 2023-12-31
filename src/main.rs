#[allow(unused_imports)]
use hudsucker::{
    async_trait::async_trait,
    certificate_authority::OpensslAuthority,
    hyper::{Body, Request, Response},
    openssl::{hash::MessageDigest, pkey::PKey, x509::X509},
    tokio_tungstenite::tungstenite::Message,
    *,
};
use pipeline::{Filter, Logger, OutboundProcessor, Pipeline};
use std::net::SocketAddr;

pub mod pipeline;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    let private_key_bytes: &[u8] = include_bytes!("../ca/httpeeker.key");
    let ca_cert_bytes: &[u8] = include_bytes!("../ca/httpeeker.cer");
    let private_key =
        PKey::private_key_from_pem(private_key_bytes).expect("Failed to parse private key");
    let ca_cert = X509::from_pem(ca_cert_bytes).expect("Failed to parse CA certificate");

    let ca = OpensslAuthority::new(private_key, ca_cert, MessageDigest::sha256(), 1_000);

    let processors: Vec<Box<dyn OutboundProcessor + Sync + Send>> = vec![
        Box::new(Filter {
            filter_string: "google.com".to_string(),
        }),
        Box::new(Logger),
    ];

    let pipeline = Pipeline::new(processors, vec![]);

    let proxy = Proxy::builder()
        .with_addr(SocketAddr::from(([127, 0, 0, 1], 3000)))
        .with_rustls_client()
        .with_ca(ca)
        .with_http_handler(pipeline)
        .build();

    if let Err(e) = proxy.start(shutdown_signal()).await {
        println!("{}", e);
    }
}
