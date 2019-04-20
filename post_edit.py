from argparse import ArgumentParser
from datetime import datetime
from os import path, remove

METADATA = """---
layout: {layout}
title: {title}
categories: {categories}
tags: {tags}
date: {date}
---
"""

now = datetime.strftime(datetime.now(), '%Y-%m-%d')
parser = ArgumentParser(prog="Post-Edit")
parser.add_argument('file', type=str, help='md file name')
parser.add_argument(
    '--date', help='specify date, default, now', default=now, type=str)
parser.add_argument('--tags', action='append', type=str, help='add tags for post')
parser.add_argument('--category', action='append', type=str,
                    help='category for post')
parser.add_argument('--title', type=str, help='title of post')
parser.add_argument('--remove', action='store_true', default=False)
args = parser.parse_args()

with open(args.file, 'r', encoding='utf-8', errors='ignore') as f:
    content = [i.rstrip() for i in f.readlines()]


metadata = {
    'layout': 'post',
    'title': args.title or content[0],
    'categories': '[' + ', '.join(args.category) + ']',
    'tags': '[' + ', '.join(args.tags) + ']',
    'date': args.date
}

dir_name, file = path.split(args.file)
new_file = path.join(dir_name, metadata['date'] + '-' + file.replace(' ', '-'))
with open(new_file, 'w', encoding='utf-8', errors='ignore') as f:
    meta_contents = METADATA.format(**metadata).split('\n')
    f.write('\n'.join(meta_contents + content))

if args.remove:
    remove(args.file)
