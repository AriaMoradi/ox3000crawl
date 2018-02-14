#![feature(conservative_impl_trait)]
#![feature(try_trait)]
// #![feature(nll)]
#![feature(underscore_lifetimes)]

extern crate futures;
extern crate hyper;
extern crate scraper;
extern crate tokio_core;

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

fn entry_uris(handle: &Handle) -> impl Stream<Item = String, Error = hyper::Error> {
    let client = Client::new(&handle);

    let uris = ENTRY_GROUPS
        .iter()
        .map(|suffix| ENTRY_GROUPS_URL_PREFIX.to_string() + suffix)
        .map(|uri| uri.parse::<Uri>().unwrap());

    let jobs = uris.map(|uri| {
        stream::unfold(Some(uri), |uri| {
            uri.map(|uri| {
                eprintln!("get: {}", uri);
                client
                    .get(uri.clone())
                    .map(move |res| {
                        eprintln!("Downloaded page from {}\nResponse: {}", uri, res.status());
                        res
                    })
                    .and_then(|res| res.body().concat2())
                    .map(|body| {
                        let doc = Html::parse_document(&*String::from_utf8_lossy(&*body));
                        entry_uris_from_body(&doc)
                    })
            })
        }).into_future()
    });

    stream::futures_unordered(jobs)
        .map(|(u, s)| stream::once(Ok(u.unwrap())).chain(s))
        .flatten()
        .map(stream::iter_ok)
        .flatten()
    // stream::iter_ok(uris)
    //     .map(move |uri: Uri| {
    //         println!("get: {}", uri);
    //         client.get(uri.clone()).map(move |res| {
    //             eprintln!("Downloaded page from {}\nResponse: {}", uri, res.status());
    //             res
    //         })
    //     })
    //     .buffer_unordered(10)
    //     .and_then(|res| res.body().concat2())
    //     .map(|body| {
    //         let doc = Html::parse_document(&*String::from_utf8_lossy(&*body));
    //         entry_uris_from_body(&doc)
    //     })
    //     .map(stream::iter_ok)
    //     .flatten()
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let euris = entry_uris(&handle);

    let res = core.run(euris.collect());
    println!("{:?}", res.unwrap().len())
}
