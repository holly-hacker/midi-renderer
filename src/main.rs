use std::{fs::File, path::PathBuf, sync::Arc};

use argh::FromArgs;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

#[derive(FromArgs)]
/// Render MIDI files to .wav files
struct CliArgs {
    /// the midi file (.mid) to render
    #[argh(option, short = 'i')]
    midi_file: PathBuf,

    /// the sound font file (.sf2) to use
    #[argh(option, short = 's')]
    soundfont_file: PathBuf,

    /// where to write the output file (.wav)
    #[argh(option, short = 'o')]
    output_file: Option<PathBuf>,

    /// the sample rate of the output file
    #[argh(option, short = 'r', default = "48000")]
    sample_rate: u32,

    /// the bit depth of the output file (one of 8, 16, 24 or 32)
    #[argh(option, short = 'd', default = "24")]
    bit_depth: u16,
}

fn main() {
    let args = argh::from_env::<CliArgs>();

    if !matches!(args.bit_depth, 8 | 16 | 24 | 32) {
        panic!(
            "Expected bit depth to be 8, 16, 24 or 32 (found {})",
            args.bit_depth
        );
    }

    println!("Reading MIDI file");
    let midi_file = {
        let mut file = File::open(&args.midi_file).expect("open midi");
        MidiFile::new(&mut file).expect("read midi")
    };
    let midi_file = Arc::new(midi_file);

    println!("Reading sf2 file");
    let sound_font = {
        let mut file = File::open(&args.soundfont_file).expect("open soundfont");
        SoundFont::new(&mut file).expect("read soundfont")
    };
    let sound_font = Arc::new(sound_font);

    println!("Initializing synth");
    let settings = SynthesizerSettings::new(args.sample_rate as i32);
    let synth = Synthesizer::new(&sound_font, &settings).expect("create synth");

    println!("Initializing sequencer");
    let mut sequencer = MidiFileSequencer::new(synth);
    sequencer.play(&midi_file, false);

    let sample_count = (settings.sample_rate as f64 * midi_file.get_length()) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    println!("Rendering to buffer");
    sequencer.render(&mut left[..], &mut right[..]);

    println!("Wrapping in wav container");
    let rendered = wrap_as_wav(
        left.into_iter().zip(right),
        settings.sample_rate as u32,
        args.bit_depth,
    );

    let output = args.output_file.unwrap_or_else(|| {
        let mut path = args.midi_file.clone();
        path.set_extension("wav");
        path
    });

    std::fs::write(output, rendered).expect("write output file");
}

pub fn wrap_as_wav(
    samples: impl Iterator<Item = (f32, f32)> + Clone,
    sample_rate: u32,
    bit_depth: u16,
) -> Vec<u8> {
    // See: http://soundfile.sapp.org/doc/WaveFormat/

    debug_assert_eq!(bit_depth % 8, 0, "Bit depth must be a multiple of 8");
    let byte_depth = bit_depth / 8;

    let mut out = vec![];

    let sample_count = samples.clone().count() as u32;
    let expected_data_length = sample_count * 2 * byte_depth as u32;

    // RIFF header
    out.extend(b"RIFF"); // ChunkID
    out.extend((36 + expected_data_length).to_le_bytes()); // ChunkSize
    out.extend(b"WAVE"); // Format
    debug_assert_eq!(12, out.len(), "length mismatch after header");

    // subchunk 1: 'fmt '
    out.extend(b"fmt "); // Subchunk1ID
    out.extend(16u32.to_le_bytes()); // Subchunk1Size
    out.extend(1u16.to_le_bytes()); // AudioFormat (1 = PCM)
    out.extend(2u16.to_le_bytes()); // NumChannels (2 for stereo)
    out.extend(sample_rate.to_le_bytes()); // SampleRate
    out.extend((sample_rate * 2 * byte_depth as u32).to_le_bytes()); // ByteRate, SampleRate * NumChannels * ByteDepth
    out.extend((2u16 * byte_depth).to_le_bytes()); // BlockAlign, NumChannels * ByteDepth
    out.extend(bit_depth.to_le_bytes()); // BitsPerSample
    // extra parameters would go here if not PCM
    debug_assert_eq!(36, out.len(), "length mismatch after subchunk 1");

    // subchunk 2: 'data'
    out.extend(b"data");
    out.extend(expected_data_length.to_le_bytes());
    for (l, r) in samples {
        // convert to 64-bit float to ensure no accuracy loss
        let (l, r) = (l as f64, r as f64);
        match bit_depth {
            8 => {
                let (l, r) = ((l + 1.) / 2., (r + 1.) / 2.);
                out.extend(((l * 256.) as u8).to_le_bytes());
                out.extend(((r * 256.) as u8).to_le_bytes());
            }
            16 => {
                out.extend(((l * 32_767.) as i16).to_le_bytes());
                out.extend(((r * 32_767.) as i16).to_le_bytes());
            }
            24 => {
                let convert = |num: i32| {
                    let bytes = num.to_le_bytes();
                    let fixed_byte_2 = (bytes[2] & 0b0111_1111) | (bytes[3] & 0b1000_0000);
                    [bytes[0], bytes[1], fixed_byte_2]
                };

                out.extend(convert((l * 8_388_607.) as i32));
                out.extend(convert((r * 8_388_607.) as i32));
            }
            32 => {
                out.extend(((l * 2_147_483_647.) as i32).to_le_bytes());
                out.extend(((r * 2_147_483_647.) as i32).to_le_bytes());
            }
            _ => unreachable!("Unexpected bit depth {bit_depth}, expected 8, 16, 24 or 32"),
        };
    }
    debug_assert_eq!(
        44 + expected_data_length as usize,
        out.len(),
        "length mismatch after subchunk 2"
    );

    out
}
