#![feature(conservative_impl_trait)]
#![feature(try_trait)]
#![feature(use_nested_groups)]

extern crate futures;
extern crate hyper;
extern crate kuchiki;
extern crate tokio_core;

// use std::io::{self, Write};
use futures::{Future, Stream, stream::futures_unordered};
use hyper::Client;
use tokio_core::reactor::Core;
use kuchiki::parse_html;

const ENTRY_GROUPS_URL_PREFIX: &str =
    "http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000_";
const ENTRY_GROUPS: &[&str] = &[
    "A-B", "C-D", "E-G", "H-K", "L-N", "O-P", "Q-R", "S", "T", "U-Z"
];

// fn entrie_uris_from_body(body: Html) -> impl Iterator {}

fn entry_uris() {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let uris = ENTRY_GROUPS
        .iter()
        .map(|suffix| ENTRY_GROUPS_URL_PREFIX.to_string() + suffix)
        .map(|uri| uri.parse().unwrap());
    let work = uris.map(|uri| {
        client.get(uri).and_then(|res| {
            println!("Response: {}", res.status());
            res.body()
                .concat2()
                .and_then(move |body| parse_html().from_utf8().from_iter(body))
        })
    });
    let list_stream = futures_unordered(work);

    let res = core.run(list_stream.collect());
    println!("{:?}", res)
}

fn main() {
    entry_uris()
    // let entries = get_entries();
}
