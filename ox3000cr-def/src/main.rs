#![feature(conservative_impl_trait)]
#![feature(try_trait)]
// #![feature(nll)]
#![feature(underscore_lifetimes)]
#![feature(proc_macro, generators)]

extern crate futures_await as futures;
extern crate hyper;
extern crate hyper_tls;
extern crate scraper;
extern crate tokio_core;

use futures::prelude::*;
use futures::{stream, Future, Stream};
use hyper::{Client, Uri};
use tokio_core::reactor::{Core, Handle};
use scraper::{Html, Selector};

const ENTRY_GROUPS_URL_PREFIX: &str =
    "http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000_";
const ENTRY_GROUPS: &[&str] = &[
    "A-B", "C-D", "E-G", "H-K", "L-N", "O-P", "Q-R", "S", "T", "U-Z"
];

fn entry_uris_from_body(body: &Html) -> (Vec<String>, Option<Uri>) {
    let select_word = Selector::parse("[title~=definition]").unwrap();
    let uris = body.select(&select_word)
        .map(|m| m.value().attr("href").unwrap().to_string())
        .collect();

    let select_next = Selector::parse("#paging a").unwrap();
    let link = body.select(&select_next)
        .filter(|m| m.text().next().unwrap() == ">")
        .next()
        .map(|m| m.value().attr("href").unwrap().parse().unwrap());

    (uris, link)
}

#[async]
fn pages_uri(uri: Uri, handle: Handle) -> hyper::Result<(Vec<String>, Option<Uri>)> {
    let connector = hyper_tls::HttpsConnector::new(4, &handle).unwrap();
    let client = Client::configure().connector(connector).build(&handle);
    eprintln!("Get: {}", uri);
    let res = await!(client.get(uri.clone()))?;
    eprintln!("Downloaded page from {}\nResponse: {}", uri, res.status());
    let body = await!(res.body().concat2())?;
    let doc = Html::parse_document(&*String::from_utf8_lossy(&*body));
    Ok(entry_uris_from_body(&doc))
}

fn entry_uris<'a>(handle: &'a Handle) -> impl Stream<Item = String, Error = hyper::Error> + 'a {
    let uris = ENTRY_GROUPS
        .iter()
        .map(|suffix| ENTRY_GROUPS_URL_PREFIX.to_string() + suffix)
        .map(|uri| uri.parse::<Uri>().unwrap());

    let jobs = uris.map(|uri| {
        stream::unfold(Some(uri), move |uri| {
            uri.map(|uri| pages_uri(uri, handle.clone()))
        }).concat2()
            .map(stream::iter_ok)
    });

    stream::futures_unordered(jobs).flatten()
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let euris = entry_uris(&handle);

    let res = core.run(euris.collect());
    println!("{:?}", res.unwrap().len())
}
