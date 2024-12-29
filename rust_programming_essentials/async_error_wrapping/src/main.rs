use std::fmt;
// use std::fs::File;
// use std::io;
// use std::io::{self, Read};
use tokio::fs::File as AsyncFile;
use tokio::io::{self as async_io, AsyncReadExt};

// Define a custom error type
#[derive(Debug)]
enum MyError {
    // Io(io::Error),
    AsyncIo(async_io::Error),
    Parse(std::num::ParseIntError),
}

// Implement `From` trait to convert `io::Error`, `async_io::Error`, and `ParseIntError` into `MyError`
// impl From<io::Error> for MyError {
//     fn from(error: io::Error) -> Self {
//         MyError::Io(error)
//     }
// }

impl From<async_io::Error> for MyError {
    fn from(error: async_io::Error) -> Self {
        MyError::AsyncIo(error)
    }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(error: std::num::ParseIntError) -> Self {
        MyError::Parse(error)
    }
}

// Implement `Display` for `MyError`
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // MyError::Io(err) => write!(f, "IO error: {}", err),
            MyError::AsyncIo(err) => write!(f, "Async IO error: {}", err),
            MyError::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

// An asynchronous function that may return an error
async fn read_and_parse_file_async(path: &str) -> Result<i32, MyError> {
    let mut file = AsyncFile::open(path).await?; // `?` propagates `async_io::Error` as `MyError`
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let number: i32 = contents.trim().parse()?; // `?` propagates `ParseIntError` as `MyError`
    Ok(number)
}

#[tokio::main]
async fn main() {
    match read_and_parse_file_async("numbers.txt").await {
        Ok(number) => println!("The number is: {}", number),
        Err(e) => println!("An error occurred: {}", e),
    }
}
