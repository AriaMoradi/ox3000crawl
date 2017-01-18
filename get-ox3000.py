#! /usr/bin/python3

from lxml import html, etree
import requests, sys

def get_entries():
    """Get all the Oxford 3000 (R) wrods from http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000 and links to their definition."""
    # make links of every page groups
    entry_groups = ['A-B', 'C-D', 'E-G', 'H-K', 'L-N', 'O-P', 'Q-R', 'S', 'T', 'U-Z']
    entry_groups_url_prefix = 'http://www.oxfordlearnersdictionaries.com/wordlist/english/oxford3000/Oxford3000_'
    entry_groups_urls = map(lambda e: entry_groups_url_prefix + e, entry_groups)

    # entries contains word and link
    entries = []
    for content_url in entry_groups_urls:
        next_link = [content_url]
        # if we have next page
        while len(next_link) > 0:
            # download it
            print("downloading page from %s ..." % next_link[0], file=sys.stderr)
            content = html.fromstring(requests.get(next_link[0]).text)

            # get all the words in this page
            entries_url = content.cssselect('[title~=definition]')
            entries_url = list(map(lambda e: (e.text, e.get('href')), entries_url))
            entries.extend(entries_url)

            # check if we have a '>' link to the next page
            links = content.cssselect('#paging a')
            next_link = map(lambda e: e.get('href'), filter(lambda e: e.text == '>', links))
            next_link = list(next_link)
    return entries

def innerHTML(sentence):
    return (sentence.text or '') + \
            ''.join([etree.tostring(child).decode('utf-8') for child in sentence.getchildren()])

def get_definitions(link):
    print("downloading page from %s ..." % link, file=sys.stderr)
    content = html.fromstring(requests.get(link).text)
    ox3000defs = content.cssselect('.sn-gs > [ox3000=y]')
    defhtml = '<ol> '
    for defitem in ox3000defs:
        # get definition of this item
        defelems = defitem.cssselect('.def')
        if len(defelems) > 1: print('Def from', link, 'has multi .def element', file=sys.stderr)
        definition = defelems[0].text

        examples = defitem.cssselect('.sn-g > .x-gs > .x-g .x')
        examhtml = '<ul class = "eg"> '
        for sentence in examples:
            examhtml += '<li> ' + innerHTML(sentence) + ' </li> '
        examhtml += '</ul> '

        defhtml += '<li> '
        defhtml += '<div class="def">' + definition + '</div> '
        defhtml += examhtml
        defhtml += '</li> '
    defhtml += '</ol> '
    return defhtml

def main():
    entries = get_entries()
    for (word, link) in entries:
        definitions = get_definitions(link)
        print(word, link, definitions, sep='\t', end='')
        

if __name__ == '__main__':
    main()
