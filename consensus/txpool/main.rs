// CITA
// Copyright 2016-2017 Cryptape Technologies LLC.

// This program is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any
// later version.

// This program is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![allow(unused_must_use, unused_variables, dead_code, unreachable_patterns, unused_imports)]
#![feature(custom_attribute)]
extern crate bincode;
extern crate dotenv;
extern crate engine;
extern crate error;
extern crate libproto;
#[macro_use]
extern crate log;
extern crate logger;
extern crate protobuf;
extern crate pubsub;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate threadpool;
extern crate tx_pool;
extern crate util;

mod candidate_pool;
mod dispatch;
mod cmd;
use candidate_pool::*;
use libproto::{key_to_id, parse_msg};
use log::LogLevelFilter;
use pubsub::start_pubsub;

use std::sync::mpsc::channel;
use std::thread;
use threadpool::ThreadPool;

const THREAD_POOL_NUMBER: usize = 2;

fn main() {
    dotenv::dotenv().ok();
    // Always print backtrace on panic.
    ::std::env::set_var("RUST_BACKTRACE", "1");
    logger::init_config("cita-consensus");
    info!("CITA:txpool");
    let (tx_sub, rx_sub) = channel();
    let (tx_pub, rx_pub) = channel();
    let (tx, rx) = channel();
    let keys = vec![
        "net.*",
        "consensus_cmd.default",
        "consensus.blk",
        "chain.richstatus",
        "jsonrpc.new_tx",
    ];
    let pool = ThreadPool::new(THREAD_POOL_NUMBER);
    start_pubsub("consensus", keys, tx_sub, rx_pub);
    thread::spawn(move || {
        loop {
            let sender = tx.clone();
            let (key, body) = rx_sub.recv().unwrap();
            pool.execute(move || {
                let (cmd_id, origin, content) = parse_msg(&body);
                sender
                    .send((key_to_id(&key), cmd_id, origin, content))
                    .unwrap();
            });
        }
    });

    let mut candidate_pool = CandidatePool::new(tx_pub.clone());
    loop {
        dispatch::dispatch(&mut candidate_pool, &rx);
    }
}
