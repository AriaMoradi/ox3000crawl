#![feature(conservative_impl_trait)]
#![allow(dead_code)]
extern crate scraper;

use scraper::Html;

const entry_groups_url_prefix: &str = "http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000_";
const entry_groups: &[&str] = &["A-B", "C-D", "E-G", "H-K", "L-N", "O-P", "Q-R", "S", "T", "U-Z"];

fn get_entries() -> impl Iterator<Item=(String, String, Html)> {
    vec![].into_iter()
}

fn main() {
    let entries = get_entries();
}
