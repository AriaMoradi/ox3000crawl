#![feature(conservative_impl_trait)]
#![feature(try_trait)]
// #![feature(nll)]
#![feature(underscore_lifetimes)]

extern crate futures;
extern crate hyper;
extern crate scraper;
extern crate tokio_core;

use futures::{stream, Future, Stream};
use hyper::{Client, Uri, client::HttpConnector};
use tokio_core::reactor::Core;
use scraper::{Html, Selector};

const ENTRY_GROUPS_URL_PREFIX: &str =
    "http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000_";
const ENTRY_GROUPS: &[&str] = &[
    "A-B", "C-D", "E-G", "H-K", "L-N", "O-P", "Q-R", "S", "T", "U-Z"
];

fn entry_uris_from_body(body: &Html) -> Vec<String> {
    let select_word = Selector::parse("[title~=definition]").unwrap();
    body.select(&select_word)
        .map(|m| m.value().attr("href").unwrap().to_string())
        .collect()
}

fn entry_uris(client: Client<HttpConnector>) -> impl Stream<Item = String, Error = hyper::Error> {
    let uris = ENTRY_GROUPS
        .iter()
        .map(|suffix| ENTRY_GROUPS_URL_PREFIX.to_string() + suffix)
        .map(|uri| uri.parse().unwrap());

    stream::iter_ok(uris)
        .map(move |uri: Uri| {
            println!("get: {}", uri);
            client.get({ uri.clone() }).map(move |res| {
                eprintln!("Downloaded page from {}\nResponse: {}", uri, res.status());
                res
            })
        })
        .buffer_unordered(10)
        .and_then(|res| res.body().concat2())
        .map(|body| {
            let doc = Html::parse_document(&*String::from_utf8_lossy(&*body));
            entry_uris_from_body(&doc)
        })
        .map(stream::iter_ok)
        .flatten()
}

fn main() {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let euris = entry_uris(client);
    // let entries = get_entries();

    let res = core.run(euris.collect());
    println!("{:?}", res.unwrap().len())
}
