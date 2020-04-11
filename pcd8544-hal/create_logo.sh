# How to create the Rust logo that displays well on the Nokia 5110 display:
convert -resize 56x48! -gravity center -extent 84x48 -depth 1 -rotate 270 -flop Rust_programming_language_black_logo.svg rust.pbm

# strip header

# reverse file using python:
# open('out.bin', 'wb').write( open('in.bin').read()[::-1] )

# Notes:
# - display is 84x48 and we want the logo centered
# - pixel aspect ratio is about 6:7, so stretch to 56x48
# - the pixels are drawn up-to-down, left-to-right due to PCD8544 protocol
# - first bit is displayed topmost, ie. least significant bit first inside byte (due to PCD8544)
#
# How this is done:
# 1. Resize to 56x48 with stretching.
# 2. Pad to 84x48 with centering.
# 3. Rotate left and flip horizontally.
# 4. Reverse the whole file bytewise.
# 5. Strip header, ie. only keep first 6*84 bytes.
