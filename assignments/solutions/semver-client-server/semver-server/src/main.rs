use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::{convert::TryInto, io};
use std::{env, io::prelude::*};

use semver::EnumRepository;
use semver_api::{ApiError, Command};

fn main() -> io::Result<()> {
    let port = env::var("PORT").unwrap_or("7878".to_string());
    let addr = format!("127.0.0.1:{}", port);
    println!("serving at {}", addr);
    let listener = TcpListener::bind(addr)?;

    let mut repository = EnumRepository::new();

    for connection in listener.incoming() {
        let mut stream = match connection {
            Ok(stream) => stream,
            Err(_) => {
                // unreachable as per documentation
                continue;
            }
        };

        let result = handle(&mut stream, &mut repository);
        if result.is_err() {
            eprintln!("Error occurred: {:?}", result);
        }

        let response: Result<String, _> = semver_api::ApiResult(result).try_into();
        match response {
            Ok(response) => {
                let _ = write!(stream, "{}", response);
            }
            Err(e) => {
                eprintln!("Error occurred: {:?}", e);
            }
        };
    }

    Ok(())
}

fn handle(
    stream: &mut TcpStream,
    repository: &mut EnumRepository,
) -> Result<Option<String>, ApiError> {
    let command = read_command(stream)?;

    let response = match command {
        Command::Get(crate_name) => {
            let crt = repository
                .get(&crate_name)
                .map_err(|e| ApiError::Underlying(e))?;
            let s: Result<String, _> = crt.try_into().map_err(|_| ApiError::Internal);
            s.map(|data| Some(data))
        }
        Command::Put(crt) => {
            repository.insert(crt);
            Ok(None)
        }
        Command::Update(update) => {
            repository.add_release(update.crate_name, update.version)?;
            Ok(None)
        }
    };
    response
}

fn read_command(stream: &mut TcpStream) -> Result<Command, ApiError> {
    let mut read_buffer = String::new();
    let mut buffered_stream = BufReader::new(stream);
    buffered_stream
        .read_line(&mut read_buffer)
        .map_err(|_| ApiError::InvalidCommand)?;
    read_buffer.as_str().try_into()
}
