use std::io::{stdout, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{fmt, thread};
use threadpool::ThreadPool;

use rand::Rng;
use serde::Serialize;

const MAP_WIDTH: i32 = 20;
const MAP_HEIGHT: i32 = 10;
const MAX_NUM_AIRCRAFTS: i32 = 10;
const MIN_NUM_AIRCRAFTS: i32 = 10;

#[derive(Clone, Debug, Serialize)]
enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::N => write!(f, "↑ "),
            Direction::NE => write!(f, "↗ "),
            Direction::E => write!(f, "→ "),
            Direction::SE => write!(f, "↘︎ "),
            Direction::S => write!(f, "↓ "),
            Direction::SW => write!(f, "↙ "),
            Direction::W => write!(f, "← "),
            Direction::NW => write!(f, "↖︎ "),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct Flight {
    id: String,
    x: i32,
    y: i32,
    direction: Direction,
}

fn main() {
    let mut traffic_data: Vec<Flight> = Vec::new();

    generate_map(&mut traffic_data);
    //dbg!(&traffic_data);
    draw_char_map(&traffic_data);

    // create a channel to communicate between the main thread and the aircraft movement thread
    // types will be infered when passed to the `process_stream` function
    // - a pair of sender and receiver for requests
    // - a pair of sender and receiver for data
    let (req_tx, req_rx) = mpsc::channel();
    let (data_tx, data_rx) = mpsc::channel();
    // periodically move the aircrafts in a thread
    let handle = thread::spawn(move || {
        let mut skip_counter = 0;
        loop {
            // check if there is a request for the latest traffic data
            if let Ok(_) = req_rx.try_recv() {
                data_tx.send(traffic_data.clone()).unwrap();
            }

            if skip_counter == 3 {
                move_aircrafts(&mut traffic_data);
                // draw_char_map(&traffic_data);
                skip_counter = 0;
            } else {
                skip_counter += 1;
            }

            sleep(Duration::from_millis(300));
        }
    });

    // other code to run...

    // now run the REST API server
    let listener = TcpListener::bind("localhost:3000").expect("Could not bind to address");

    println!("Server listening on port 3000");

    // Use a thread pool to handle incoming connections
    let thread_pool = ThreadPool::new(4);
    let data_mutex = Arc::new(Mutex::new(data_rx));

    for stream_result in listener.incoming() {
        let req_tx_clone = req_tx.clone();
        let data_mutex_clone = data_mutex.clone();
        if let Ok(stream) = stream_result {
            // Passing the cloned ones to individual threads
            thread_pool.execute(move || {
                process_stream(stream, &req_tx_clone, data_mutex_clone);
            });
        }
    }
    // This is handling incoming connection with single thread
    // for stream_result in listener.incoming() {
    //     if let Ok(stream) = stream_result {
    //         process_stream(stream, &req_tx, &data_rx);
    //     }
    // }

    handle.join().unwrap();
}

fn add_new_flight(data_set: &mut Vec<Flight>) {
    let mut rng = rand::thread_rng();
    let letter1: char = rng.gen_range(b'A'..b'Z') as char;
    let letter2: char = rng.gen_range(b'A'..b'Z') as char;
    let number: u32 = rng.gen_range(10..9999);
    let new_id = format!("{}{}{:02}", letter1, letter2, number);

    // generate random x, y coordinates
    let new_x = rand::thread_rng().gen_range(0..MAP_WIDTH);
    let new_y = rand::thread_rng().gen_range(0..MAP_HEIGHT);

    // generate a random direction
    let dir = rand::thread_rng().gen_range(0..8);
    let new_dir = match dir {
        0 => Direction::N,
        1 => Direction::NE,
        2 => Direction::E,
        3 => Direction::SE,
        4 => Direction::S,
        5 => Direction::SW,
        6 => Direction::W,
        7 => Direction::NW,
        _ => Direction::N,
    };

    data_set.push(Flight {
        id: new_id,
        x: new_x,
        y: new_y,
        direction: new_dir,
    });
}

fn draw_char_map(data_set: &[Flight]) {
    let mut lock = stdout().lock();
    for y in 0..(MAP_HEIGHT) {
        write!(lock, " ").unwrap();
        for _ in 0..(MAP_WIDTH) {
            write!(lock, "-- ").unwrap();
        }
        write!(lock, "\r\n").unwrap();
        for x in 0..(MAP_WIDTH) {
            write!(lock, "|").unwrap();
            // is there an aircraft in this box's coordinates?
            let ufo = data_set
                .iter()
                .find(|flight| flight.x == x && flight.y == y);
            match ufo {
                None => write!(lock, "  ").unwrap(),
                Some(f) => write!(lock, "{}", f.direction.to_string()).unwrap(),
            }
        }
        write!(lock, "|\r\n").unwrap();
    }
    // print the bottom line
    for _ in 0..(MAP_WIDTH) {
        write!(lock, " --").unwrap();
    }
    write!(lock, "\r\n").unwrap();
}

fn generate_map(data_set: &mut Vec<Flight>) {
    let num_aircrafts = rand::thread_rng().gen_range(MIN_NUM_AIRCRAFTS..(MAX_NUM_AIRCRAFTS + 1));
    for _ in 0..num_aircrafts {
        add_new_flight(data_set);
    }
}

fn move_aircrafts(data_set: &mut [Flight]) {
    for i in 0..data_set.iter().count() {
        match &data_set[i].direction {
            Direction::N => {
                data_set[i].y = data_set[i].y - 1;
                if data_set[i].y < 0 {
                    data_set[i].y = MAP_HEIGHT - 1;
                }
            }

            Direction::NE => {
                data_set[i].y = data_set[i].y - 1;
                if data_set[i].y < 0 {
                    data_set[i].y = MAP_HEIGHT - 1;
                }
                data_set[i].x = data_set[i].x + 1;
                if data_set[i].x >= MAP_WIDTH {
                    data_set[i].x = 0;
                }
            }

            Direction::E => {
                data_set[i].x = data_set[i].x + 1;
                if data_set[i].x >= MAP_WIDTH {
                    data_set[i].x = 0;
                }
            }

            Direction::SE => {
                data_set[i].x = data_set[i].x + 1;
                if data_set[i].x >= MAP_WIDTH {
                    data_set[i].x = 0;
                }
                data_set[i].y = data_set[i].y + 1;
                if data_set[i].y >= MAP_HEIGHT {
                    data_set[i].y = 0;
                }
            }

            Direction::S => {
                data_set[i].y = data_set[i].y + 1;
                if data_set[i].y >= MAP_HEIGHT {
                    data_set[i].y = 0;
                }
            }

            Direction::SW => {
                data_set[i].y = data_set[i].y + 1;
                if data_set[i].y >= MAP_HEIGHT {
                    data_set[i].y = 0;
                }
                data_set[i].x = data_set[i].x - 1;
                if data_set[i].x < 0 {
                    data_set[i].x = MAP_WIDTH - 1;
                }
            }

            Direction::W => {
                data_set[i].x = data_set[i].x - 1;
                if data_set[i].x < 0 {
                    data_set[i].x = MAP_WIDTH - 1;
                }
            }

            Direction::NW => {
                data_set[i].x = data_set[i].x - 1;
                if data_set[i].x < 0 {
                    data_set[i].x = MAP_WIDTH - 1;
                }
                data_set[i].y = data_set[i].y - 1;
                if data_set[i].y < 0 {
                    data_set[i].y = MAP_HEIGHT - 1;
                }
            }
        }
    }
}

fn process_stream(
    mut stream: TcpStream,
    data_requester: &Sender<()>,
    // this is for single thread
    // data_receiver: &Receiver<Vec<Flight>>,
    data_receiver: Arc<Mutex<Receiver<Vec<Flight>>>>,
) {
    let http_request = read_http_request(&mut stream);
    // filter requests

    if http_request.is_empty() {
        return;
    }

    // http requests starts with a method and a path
    if http_request.len() < 6 {
        return;
    }

    // method must be GET
    let test = &http_request[0][..6];
    if test != "GET / " {
        println!("Invalid request ignored: {}", http_request[0]);
        return;
    }
    let latest_traffic_data = get_latest_traffic_data(data_requester, data_receiver);
    dbg!(&latest_traffic_data);
    send_http_respond(&mut stream, &latest_traffic_data);
}

fn read_http_request(stream: &mut TcpStream) -> Vec<String> {
    let buf_reader = std::io::BufReader::new(stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("HTTP request: {:#?}", http_request);

    http_request
}

fn send_http_respond(stream: &mut TcpStream, data: &Option<Vec<Flight>>) {
    let respond_line = "HTTP/1.1 200 OK";

    // let payload = "<h1>Hello Client Application</h1>\r\n";
    // Serialize the data to JSON
    let empty: Vec<Flight> = vec![];
    let data_unwrapped = match data {
        None => &empty,
        Some(data) => data,
    };
    let serialized_data = serde_json::to_string(data_unwrapped);

    // define payload
    let payload = match serialized_data {
        Ok(data) => data,
        _ => "[]".to_string(),
    };

    let content_length = payload.len();
    // let content_type = "text/html";
    let content_type = "application/json";
    let headers = format!(
        "Content-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n",
        content_type, content_length
    );

    let http_response = format!("{}\r\n{}\r\n{}\r\n", respond_line, headers, payload);

    stream.write_all(http_response.as_bytes()).unwrap();
}

fn get_latest_traffic_data(
    data_requester: &Sender<()>,
    // this is for single thread
    // data_receiver: &Receiver<Vec<Flight>>,
    data_receiver: Arc<Mutex<Receiver<Vec<Flight>>>>,
) -> Option<Vec<Flight>> {
    data_requester.send(()).unwrap();
    // this is for single thread
    // match data_receiver.recv_timeout(Duration::from_millis(5000)) {
    match data_receiver
        .lock()
        .unwrap()
        .recv_timeout(Duration::from_millis(5000))
    {
        Ok(data) => Some(data),
        _ => None,
    }
    // automatically unlock the mutex from here
}
