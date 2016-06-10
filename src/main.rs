extern crate crypto;
extern crate rustc_serialize;
extern crate time;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;

use std::io::prelude::*;

use rustc_serialize::base64::FromBase64;

use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

use std::io;

use std::env;

use std::str;

static NTHREADS: i32 = 8;
static THREAD_WORK: i32 = 500;

fn main() {

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        let jwt_ex1 = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.4twFt5NiznN84AWoo1d7KO1T_yoc0Z6XOpOVswacPZg";    // key is "secret"
        let jwt_ex2 = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Fw4maeqOtL8pPwiI2_VzYBo4JQ91P1Ow3X3hNqx2wPg";    // key is " samantha1"
        println!("Usage is:\n{} JWT < /path/to/wordlist\n", args[0]);
        println!("Example JWTs:\n{} (key: 'secret')\n{} (key: ' samantha1')", jwt_ex1, jwt_ex2);
        return;
    }
    let ref jwt = args[1];

    let (bdata, bmac) = match jwt.rfind(".") {
        Some(split) => (
            jwt[..split].to_string(),
            jwt[split+1..].to_string()),
        _ => ("".to_string(), "".to_string())
    };

    let shared_body_b64   = Arc::new(bdata);
    let shared_hmac_bytes = Arc::new(bmac.from_base64().expect("HMAC is not valid b64"));

    let (checkin_tx, checkin_rx) = mpsc::channel();

    let mut worker_channels = vec![];
    let mut worker_threads = vec![];
    println!("Starting {} threads with work factor {}", NTHREADS, THREAD_WORK);
    for id in 0..NTHREADS {

        // The sender endpoint can be copied
        let thread_checkin_tx = checkin_tx.clone();

        // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`
        let (candidates_tx, candidates_rx): (Sender<Vec<Vec<u8>>>, Receiver<Vec<Vec<u8>>>) = mpsc::channel();

        // Store the transmitter in a vector so we can send words to the thread
        worker_channels.push(candidates_tx);

        let child_body_b64   = shared_body_b64.clone();
        let child_hmac_bytes = shared_hmac_bytes.clone();

        // Each thread will send its id via the channel
        worker_threads.push(thread::spawn(move || {
            let local_body_bytes = &child_body_b64[..].as_bytes();
            let local_hmac_bytes = &child_hmac_bytes[..];
            loop {
                let candidates = match candidates_rx.recv() {
                    Ok(w) => w,
                    _ => vec![]
                };
                // no more candidates?
                if candidates.len() == 0 {
                    break;
                }
                // process words
                let mut done = false;
                for keybytes in candidates {
                    let hmac_key = &keybytes[..];
                    let mut hmac = Hmac::new(Sha256::new(), hmac_key);
                    hmac.input(local_body_bytes);
                    if &*local_hmac_bytes == hmac.result().code() {
                        // announce that we found the key
                        let bytes: Vec<String> = keybytes.iter().map(|b| format!("{:02X}", b)).collect();

                        print!("\n\nKey found: \n{} ", bytes.join(" "));
                        match str::from_utf8(&*keybytes) {
                            Ok(word) => println!("('{}')", word),
                            _ => println!("(not valid UTF-8)"),
                        }

                        thread_checkin_tx.send(-1);
                        done = true;
                        break;
                    }
                }
                if done {
                    break;
                }
                // request more candidates to try
                thread_checkin_tx.send(id);
            }
        }));
    }

    // Read dictionary from stdin
    let input = io::stdin();
    let reader = input.lock();
    let mut reader_lines = reader.split(b'\n');

    // send an initial batch of words to each worker
    for channel in &worker_channels {
        channel.send(
            reader_lines.by_ref()
                .take(THREAD_WORK as usize)
                .map(|x| x.unwrap())
                .collect());
    }

    // Set up timing structure
    let mut start_interval_time: u64 = 0;
    let mut idx_interval_time: u64 = 0;
    let mut work_interval_target: u64 = 200;
    // whenever we hear back from a worker, send them another batch
    loop {
        let finished_id = checkin_rx.recv().expect("Failed to get finished_id");
        if finished_id == -1 {
            //println!("Key has been found, stop sending anything");
            break;
        }

        // send batch of words to thread requesting them
        worker_channels[finished_id as usize].send(
            reader_lines.by_ref()
                .take(THREAD_WORK as usize)
                .map(|x| x.unwrap())
                .collect());

        // update timing structure
        if idx_interval_time == work_interval_target {
            let hashes = THREAD_WORK * work_interval_target as i32;
            let interval_ns = time::precise_time_ns() - start_interval_time;

            // we want this to run about once per second
            let interval_recip_s = 1000000000.0 / interval_ns as f64;
            work_interval_target = (work_interval_target as f64 * interval_recip_s) as u64;
            if work_interval_target < 1 {
                work_interval_target = 200;
            }

            // display status
            let hps = hashes as f64 * interval_recip_s;
            print!("\r                  \r{} hashes per second", hps as i32);
            io::stdout().flush().ok().expect("Could not flush stdout");
            idx_interval_time = 0;
            start_interval_time = time::precise_time_ns();
        } else {
            idx_interval_time += 1;
        }
    }

}
