#! /usr/bin/python3

from lxml import html
import requests

entry_groups = ['A-B', 'C-D', 'E-G', 'H-K', 'L-N', 'O-P', 'Q-R', 'S', 'T', 'U-Z']
entry_groups_url_prefix = 'http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000_'
entry_groups_urls = map(lambda e: entry_groups_url_prefix + e, entry_groups)

dbg = []
entries = []
for content_url in entry_groups_urls:
    next_link = [content_url]
    while len(next_link) > 0:
        content = html.fromstring(requests.get(next_link[0]).text)

        entries_url = content.cssselect('[title~=definition]')
        dbg = (entries_url)
        entries_url = list(map(lambda e: (e.text, e.get('href')), entries_url))
        entries.extend(entries_url)

        links = content.cssselect('#paging a')
        next_link = map(lambda e: e.get('href'), filter(lambda e: e.text == '>', links))
        next_link = list(next_link)
