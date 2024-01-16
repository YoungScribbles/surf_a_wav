
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

struct WavFile {
    chunk_id:       u32,
    chunk_size:     u32,
    format:         u32,

    subchunk_id:    u32,
    subchunk_size:  u32,
    audio_format:   u16,
    channel_no:     u16,
    sample_rate:    u32,
    byte_rate:      u32,
    block_align:    u16,
    bit_depth:      u16,

    data_id:        u32,
    data_size:      u32,
    audio_data:     Vec<u8>,

    little_endian: bool
}

impl WavFile {

    fn read_wav(path: &str) -> WavFile {
        let mut file = File::open(path)
            .expect("Invalid path {path}");

        let mut data = vec![];
        file.read_to_end(&mut data)
            .expect("Failed to read");
        
        // Assume little endian as default
        let mut little_endian = true;
        let buffer_size = data.len();
        let mut chunk_size = bytes_to_uint(&data, 4, 4, little_endian);

        // Override to big endian
        if chunk_size + 8 != buffer_size as u32 {
            little_endian = false;
            chunk_size = bytes_to_uint(&data, 4, 4, little_endian);
        }

        let chunk_id = bytes_to_uint(&data, 0, 4, false);
        let format = bytes_to_uint(&data, 8, 4, false);
        let subchunk_id = bytes_to_uint(&data, 12, 4, false);
        let subchunk_size = bytes_to_uint(&data, 16, 4, little_endian);
        let audio_format: u16 = bytes_to_uint(&data, 20, 2, little_endian) as u16;
        let channel_no: u16 = bytes_to_uint(&data, 22, 2, little_endian) as u16;
        let sample_rate = bytes_to_uint(&data, 24, 4, little_endian);
        let byte_rate = bytes_to_uint(&data, 28, 4, little_endian);
        let block_align: u16 = bytes_to_uint(&data, 32, 2, little_endian) as u16;
        let bit_depth: u16 = bytes_to_uint(&data, 34, 2, little_endian) as u16;

        let data_id = bytes_to_uint(&data, 36, 4, false);
        let data_size = bytes_to_uint(&data, 40, 4, little_endian);
        let audio_data = data.split_off(44);

        let wav = WavFile{
            chunk_id,
            chunk_size,
            format,
        
            subchunk_id,
            subchunk_size,
            audio_format,
            channel_no,
            sample_rate,
            byte_rate,
            block_align,
            bit_depth,
        
            data_id,
            data_size,
            audio_data,
        
            little_endian
        };

        wav
    }

    fn write_to_disk(&mut self, path: &str) {
        let mut out = Vec::new();

        out.append(&mut uint_to_bytes(self.chunk_id, 4, false));
        out.append(&mut uint_to_bytes(self.chunk_size, 4, self.little_endian));
        out.append(&mut uint_to_bytes(self.format, 4, false));
        out.append(&mut uint_to_bytes(self.subchunk_id, 4, false));
        out.append(&mut uint_to_bytes(self.subchunk_size, 4, self.little_endian));
        out.append(&mut uint_to_bytes(self.audio_format as u32, 2, self.little_endian));
        out.append(&mut uint_to_bytes(self.channel_no as u32, 2, self.little_endian));
        out.append(&mut uint_to_bytes(self.sample_rate, 4, self.little_endian));
        out.append(&mut uint_to_bytes(self.byte_rate, 4, self.little_endian));
        out.append(&mut uint_to_bytes(self.block_align as u32, 2, self.little_endian));
        out.append(&mut uint_to_bytes(self.bit_depth as u32, 2, self.little_endian));
        out.append(&mut uint_to_bytes(self.data_id, 4, false));
        out.append(&mut uint_to_bytes(self.data_size, 4, self.little_endian));
        out.append(&mut self.audio_data.clone());   // Todo: Fix this
    
        std::fs::write(path, out).unwrap();
    }

    fn modify_bit_depth(&mut self, bit_depth: u16) {
        let mut modified_audio: Vec<u8> = Vec::new();
        let mut cur_bytes_per_sample = self.bit_depth / 8;
        let mut new_bytes_per_sample = bit_depth / 8;
        let mut index: usize = 0; 

        if self.bit_depth  % 8 != 0 {
            cur_bytes_per_sample += 1;
        }
        if bit_depth % 8 != 0 {
            new_bytes_per_sample += 1;
        }

        while index < self.data_size as usize {
            let val = bytes_to_uint(&self.audio_data, index, cur_bytes_per_sample as usize, 
                self.little_endian);

            modified_audio.append(&mut uint_to_bytes(
                scale_sample_bit_depth(val, self.bit_depth, bit_depth), 
                new_bytes_per_sample as usize, self.little_endian));

            index += usize::from(cur_bytes_per_sample);
        }

        self.audio_data = modified_audio;

        // Update all fields that have changed
        self.bit_depth = bit_depth;
        if self.bit_depth % 8 != 0 {
            self.bit_depth += 8 - (self.bit_depth % 8);
        }

        self.block_align = self.channel_no * (self.bit_depth / 8);
        self.byte_rate = self.sample_rate * u32::from(self.block_align);
        self.data_size = self.audio_data.len() as u32;
        self.chunk_size = self.data_size + 36;
    }

    fn report_header(&self) {
        println!("Chunk ID - {}", self.chunk_id);
        println!("Chunk size - {}", self.chunk_size);
        println!("Format - {}", self.format);
        println!("Subchunk ID - {}", self.subchunk_id);
        println!("Subchunk size - {}", self.subchunk_size);
        println!("audio format - {}", self.audio_format);
        println!("Channels - {}", self.channel_no);
        println!("Sample Rate - {}", self.sample_rate);
        println!("Byte Rate - {}", self.byte_rate);
        println!("Block Align - {}", self.block_align);
        println!("Bit Depth - {}", self.bit_depth);
        println!("Data ID - {}", self.data_id);
        println!("Data size - {}", self.data_size);
    }
}

fn main() {
    let audio_path = "./test/sample.wav";

    if Path::new(audio_path).exists() {
        //let wav = get_wav(audio_path);
        let mut wav = WavFile::read_wav(audio_path);
        wav.modify_bit_depth(8);
        wav.report_header();
        wav.write_to_disk("./test/Fixedsample.wav");
    } else {
        println!("Invalid path!");
    }
}

fn bytes_to_uint(vec: &Vec<u8>, index: usize, len: usize, lil_end: bool) -> u32 {
    // Init to 0 so we can cast down to u16 and u8
    let mut vals: [u32; 4] = [0; 4];
    let mut ret: u32 = 0;
    let mut count = 0;

    if lil_end {
        for i in 0..len {
            vals[i] = (vec[index + i]) as u32;
        }
    } else {
        for i in 0..len {
            vals[i] = (vec[index + len - 1 - i]) as u32;
        }
    }

    for i in 0..len {
        ret |= vals[i] << 8 * count;
        count += 1;
    }

    ret
}

fn uint_to_bytes(val: u32, len: usize, lil_end: bool) -> Vec<u8> {
    let mut ret: Vec<u8> = Vec::new();
    let mut count = 0;

    while count < len {
        ret.push((val >> count * 8) as u8 & 0xFF);
        count += 1;
    }

    if !lil_end {
        ret.reverse()
    }

    ret
}

fn scale_sample_bit_depth(sample: u32, old: u16, new: u16) -> u32 {
    // Wav formating requires complete bytes so one of these must always be true.
    assert!((old % 8 == 0) || (new % 8 == 0));

    let mut ret: u32;
    if old > new {
        let scaler = ipow(2, u32::from(old - new));
        ret = sample / scaler;
    } else {
        let scaler = ipow(2, u32::from(new - old));
        ret = sample * scaler;
    }

    // Because I think it is funny, a user can opt to scale for a partial byte, but to keep
    // it functional, we need to scale it back up to the nearest full byte once we are done.
    // This reduces the resolution of their audio but is a viable artistic choice.
    if new % 8 != 0 {
        ret = scale_sample_bit_depth(ret, new, new + (8 - new % 8))
    }

    ret
}

fn ipow(base: u32, exponent: u32) -> u32 {
    let mut ret: u32 = 1;
    let mut count = 0;

    while count < exponent {
        ret *= base;
        count += 1;
    }

    ret
}