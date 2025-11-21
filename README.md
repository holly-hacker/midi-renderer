# MIDI Renderer

A simple app to render a MIDI file to a .wav file, using a given sound font.

## Installation

Build a copy yourself using the rust toolchain, or install it using cargo:
```bash
cargo install --locked --git https://github.com/holly-hacker/midi-renderer
```

## Usage

Provide a MIDI file and a sound font via commandline arguments.
```sh
# writes to MyMusic.wav
midi-renderer -i MyMusic.mid -s MySoundFont.sf2
```

Alternatively, use one of the optional arguments to specify the output file, sample rate or bit depth.
```sh
midi-renderer --help
```
```
Usage: midi-renderer -i <midi-file> -s <soundfont-file> [-o <output-file>] [-r <sample-rate>] [-d <bit-depth>]

Render MIDI files to .wav files

Options:
  -i, --midi-file   the midi file (.mid) to render
  -s, --soundfont-file
                    the sound font file (.sf2) to use
  -o, --output-file where to write the output file (.wav)
  -r, --sample-rate the sample rate of the output file
  -d, --bit-depth   the bit depth of the output file (one of 8, 16, 24 or 32)
  --help, help      display usage information
```

## Attribution

This project is a simple shell built around [rustysynth](https://github.com/sinshu/rustysynth) by [Nobuaki Tanaka
](https://github.com/sinshu).
