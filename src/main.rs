extern crate crypto;
extern crate getopts;
extern crate rustc_serialize;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;

use std::io::prelude::*;
//use std::io::BufReader;
//use std::fs::File;

use rustc_serialize::base64::FromBase64;

use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

use std::io;

static NTHREADS: i32 = 4;
static THREAD_WORK: i32 = 100;

fn main() {

    // set up the dictionary reader
    /*
    // Read dictionary from file
    let path = "aux/rockyou_utf8.txt";
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => panic!("couldn't open {}", path),
    };
    let mut reader_lines = BufReader::new(file).lines();
    */
    // Read dictionary from stdin
    let input = io::stdin();
    let reader = input.lock();
    let mut reader_lines = reader.lines();

    //let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.4twFt5NiznN84AWoo1d7KO1T_yoc0Z6XOpOVswacPZg";    // key is "secret"
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzb21lIjoicGF5bG9hZCJ9.Fw4maeqOtL8pPwiI2_VzYBo4JQ91P1Ow3X3hNqx2wPg";      // key is " samantha1"
    let (bdata, bmac) = match jwt.rfind(".") {
        Some(split) => (
            &jwt[..split],
            &jwt[split+1..]),
        _ => ("", "")
    };

    let shared_body_bytes = Arc::new(bdata.as_bytes());
    let shared_hmac_bytes = Arc::new(bmac.from_base64().expect("HMAC is not valid b64"));

    let (checkin_tx, checkin_rx) = mpsc::channel();

    let mut worker_channels = vec![];
    let mut worker_threads = vec![];
    for id in 0..NTHREADS {

        // The sender endpoint can be copied
        let thread_checkin_tx = checkin_tx.clone();

        // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`
        let (words_tx, words_rx): (Sender<Vec<String>>, Receiver<Vec<String>>) = mpsc::channel();

        // Store the transmitter in a vector so we can send words to the thread
        worker_channels.push(words_tx);

        let child_body_bytes = shared_body_bytes.clone();
        let child_hmac_bytes = shared_hmac_bytes.clone();

        // Each thread will send its id via the channel
        worker_threads.push(thread::spawn(move || {
            let local_body_bytes = &child_body_bytes[..];
            let local_hmac_bytes = &child_hmac_bytes[..];
            loop {
                let words = match words_rx.recv() {
                    Ok(w) => w,
                    _ => vec![]
                };
                // no more words?
                if words.len() == 0 {
                    break;
                }
                // process words
                let mut done = false;
                for word in words {
                    let hmac_key = word.as_bytes();
                    let mut hmac = Hmac::new(Sha256::new(), hmac_key);
                    hmac.input(local_body_bytes);
                    if &*local_hmac_bytes == hmac.result().code() {
                        // announce that we found the key
                        println!("Key found: '{}'", word);
                        thread_checkin_tx.send(-1).unwrap();
                        done = true;
                        break;
                    }
                }
                if done {
                    break;
                }
                // request more candidates to try
                thread_checkin_tx.send(id).unwrap();
            }
        }));
    }

    // send an initial batch of words to each worker
    for channel in &worker_channels {
        channel.send(
            reader_lines.by_ref()
                .take(THREAD_WORK as usize)
                .map(|x| x.unwrap())
                .collect())
            .unwrap();
    }

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
                .collect())
            .unwrap();

    }

}
