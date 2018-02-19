#![feature(universal_impl_trait)]
#![feature(conservative_impl_trait)]
#![feature(try_trait)]
// #![feature(nll)]
#![feature(underscore_lifetimes)]
#![feature(proc_macro, generators)]

extern crate futures_await as futures;
extern crate hyper;
extern crate hyper_tls;
extern crate maud;
extern crate scraper;
extern crate tokio_core;
extern crate tokio_retry;

use std::io;
use futures::prelude::*;
use futures::{stream, Future, Stream};
use hyper::{Client, Uri};
use tokio_core::reactor::{Core, Handle};
use tokio_retry::{Retry, strategy::{jitter, ExponentialBackoff}};
use scraper::{Html, Selector};
use maud::html;

const ENTRY_GROUPS_URL_PREFIX: &str =
    "http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000_";
const ENTRY_GROUPS: &[&str] = &[
    "A-B", "C-D", "E-G", "H-K", "L-N", "O-P", "Q-R", "S", "T", "U-Z"
];

#[async]
fn download_doc(uri: Uri, handle: Handle) -> hyper::Result<Html> {
    let connector = hyper_tls::HttpsConnector::new(4, &handle).unwrap();
    let client = Client::configure().connector(connector).build(&handle);

    let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(3);

    eprintln!("Get: {}", uri);
    // let request = client.get(uri.clone());
    let uri_ = uri.clone();
    let res = await!(Retry::spawn(handle, retry_strategy, move || client.get(uri.clone())))
        .map_err(|err| io::Error::new(io::ErrorKind::Other, Box::new(err)))?;
    eprintln!("Downloaded page from {}\nResponse: {}", uri_, res.status());

    let body = await!(res.body().concat2())?;
    Ok(Html::parse_document(&*String::from_utf8_lossy(&*body)))
}

#[async]
fn pages_uri_and_next(uri: Uri, handle: Handle) -> hyper::Result<(Vec<String>, Option<Uri>)> {
    let body = await!(download_doc(uri, handle))?;

    let select_word = Selector::parse("[title~=definition]").unwrap();
    let uris = body.select(&select_word)
        .map(|m| m.value().attr("href").unwrap().to_string())
        .collect();

    let select_next = Selector::parse("#paging a").unwrap();
    let link = body.select(&select_next)
        .filter(|m| m.text().next().unwrap() == ">")
        .next()
        .map(|m| m.value().attr("href").unwrap().parse().unwrap());

    Ok((uris, link))
}

fn entry_uris<'a>(handle: &'a Handle) -> impl Stream<Item = String, Error = hyper::Error> + 'a {
    let uris = ENTRY_GROUPS
        .iter()
        .map(|suffix| ENTRY_GROUPS_URL_PREFIX.to_string() + suffix)
        .map(|uri| uri.parse::<Uri>().unwrap());

    let jobs = uris.map(|uri| {
        stream::unfold(Some(uri), move |uri| {
            uri.map(|uri| pages_uri_and_next(uri, handle.clone()))
        }).concat2()
            .map(stream::iter_ok)
    });

    stream::futures_ordered(jobs).flatten()
}

#[async]
fn get_defs(uri: Uri, handle: Handle) -> hyper::Result<String> {
    let body = await!(download_doc(uri, handle))?;

    let select_def_items = Selector::parse(".sn-gs > .sn-g").unwrap();
    let select_ox3000_def_items = Selector::parse(".sn-gs > [ox3000=y]").unwrap();
    let select_def = Selector::parse(".def").unwrap();
    // let select_examples = Selector::parse(".sn-g > .x-gs > .x-g .x").unwrap();

    let mut defs = body.select(&select_def_items);
    let ox3000_defs = if defs.by_ref().count() == 1 {
        defs
    } else {
        body.select(&select_ox3000_def_items)
    };

    let result = html! {
        @for def_item in ox3000_defs {
            div class="def" {
                (def_item
                    .select(&select_def)
                    .next()
                    .map(|m| m.text().next().unwrap())
                    .unwrap_or(""))
            }
            li {
            }
        }
    };

    Ok(result.into_string())
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let euris = entry_uris(&handle);
    let defs = euris
        .map(|uri| uri.parse().unwrap())
        .map(|uri| get_defs(uri, handle.clone()))
        .buffered(100);

    let res = core.run(defs.take(100).collect());
    println!("{:?}", res.unwrap())
}
