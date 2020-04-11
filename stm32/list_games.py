#!/usr/bin/env python3

from os import listdir
from os.path import isfile, join


PATH = '../games/'

roms = [f for f in listdir(PATH) if isfile(join(PATH, f))]

for rom in roms:
    name = rom.upper().replace('.', '_')
    print(f'static ROM_{name}: &\'static [u8] = include_bytes!("../{PATH}{rom}");')

print()

print("let ROMS = [")
for rom in roms:
    name = rom.upper().replace('.', '_')
    print(f'    ("{name}", ROM_{name}),')
print("];")
