use std::process::ExitCode;

use std::env;
use std::io;
use std::sync::Arc;

use axum::{
    Router,
    response::{self, IntoResponse},
};

use rs_docker_images2ql::async_graphql;
use rs_docker_images2ql::bollard;

use bollard::Docker;

use async_graphql::{EmptyMutation, EmptySubscription, Schema, http::GraphiQLSource};

use async_graphql_axum::GraphQL;

use std::fs;
use tokio::net::TcpListener;

use rs_docker_images2ql::Query;

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

fn docker2schema(d: Docker) -> Schema<Query, EmptyMutation, EmptySubscription> {
    let q: Query = Query {
        docker: Arc::new(d),
    };
    Schema::build(q, EmptyMutation, EmptySubscription).finish()
}

async fn sub() -> Result<(), io::Error> {
    let listen_addr: String = env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".into());

    let sock_path: String =
        env::var("DOCKER_SOCK_PATH").unwrap_or_else(|_| "/var/run/docker.sock".into());

    let timeout_seconds: u64 = env::var("DOCKER_TIMEOUT_SECONDS")
        .unwrap_or_else(|_| "60".into())
        .parse()
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid DOCKER_TIMEOUT_SECONDS: {e}"),
            )
        })?;

    let cliver = bollard::API_DEFAULT_VERSION;

    let d: Docker = Docker::connect_with_socket(&sock_path, timeout_seconds, cliver)
        .map_err(io::Error::other)?;

    let schema: Schema<_, _, _> = docker2schema(d);
    let sdl: String = schema.sdl();

    fs::write("docker-image.graphql", sdl.as_bytes())?;

    let app = Router::new().route(
        "/",
        axum::routing::get(graphiql).post_service(GraphQL::new(schema)),
    );
    axum::serve(TcpListener::bind(listen_addr).await?, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match sub().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
