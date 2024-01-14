
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

struct WavFile {
    chunk_id: u32,
    chunk_size: u32,
    format: u32,

    subchunk_id: u32,
    subchunk_size: u32,
    audio_format: u16,
    channel_no: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bit_depth: u16,

    data_id: u32,
    data_size: u32,
    audio_data: Vec<u8>,

    little_endian: bool
}

impl WavFile {

    fn modify_bit_depth(mut self, bit_depth: u16) {

        let mut modified_audio: Vec<u8> = Vec::new();
        let mut cur_bytes_per_sample = self.bit_depth / 8;
        let mut new_bytes_per_sample = bit_depth / 8;
        let mut index: usize = 0; 

        if cur_bytes_per_sample % 8 > 0 {
            cur_bytes_per_sample += 1;
        }
        if new_bytes_per_sample % 8 > 0 {
            new_bytes_per_sample += 1;
        }

        while index < self.data_size as usize {
            let val = bytes_to_uint(&self.audio_data, index, cur_bytes_per_sample as usize, 
                self.little_endian);

            modified_audio.append(&mut uint_to_bytes(
                scale_sample_bit_depth(val, self.bit_depth, bit_depth), 
                new_bytes_per_sample as usize, self.little_endian));

            index += cur_bytes_per_sample as usize;
        }

        self.audio_data = modified_audio;

        // Update all fields that have changed
        self.bit_depth = bit_depth / 8;
        if bit_depth % 8 != 0 {
            self.bit_depth += 1;
        }

        self.block_align = self.channel_no * (self.bit_depth / 2);
        self.byte_rate = self.sample_rate * self.block_align as u32;
        self.data_size = index as u32 * (bit_depth as u32 / 8);
        self.chunk_size = self.data_size + 36;
    }

    fn write_to_disk(path: &str) {

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
        let wav = get_wav(audio_path);

        wav.report_header();

        wav.modify_bit_depth(8);
    } else {
        println!("Invalid path!");
    }
}

fn get_wav(path: &str) -> WavFile {

    let mut file = File::open(path)
        .expect("Invalid path {path}");

    let mut data = vec![];
    file.read_to_end(&mut data)
        .expect("Failed to read");

    let buffer_size = data.len();
    
    // Determine endianess
    let mut little_endian = true;
    let mut chunk_size: u32 = 0;
    chunk_size |= (data[4] as u32) << 24;
    chunk_size |= (data[5] as u32) << 16;
    chunk_size |= (data[6] as u32) << 8;
    chunk_size |= data[7] as u32;

    if chunk_size + 8 == buffer_size as u32 {
        little_endian = false;
    } else {
        chunk_size = 0;
        chunk_size |= (data[7] as u32) << 24;
        chunk_size |= (data[6] as u32) << 16;
        chunk_size |= (data[5] as u32) << 8;
        chunk_size |= data[4] as u32;
        assert!(chunk_size + 8 == buffer_size as u32);
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

fn bytes_to_uint(vec: &Vec<u8>, index: usize, len: usize, lil_end: bool) -> u32 {

    // Init to 0 so we can cast down to u16 and u8
    let mut vals: [u32; 4] = [0; 4];
    let mut ret: u32 = 0;
    let mut count = 0;

    if lil_end {
        for i in 0..len {
            vals[i] = vec[index + i] as u32;
        }
    } else {
        for i in 0..len {
            vals[i] = vec[index + len - 1 - i] as u32;
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

    // As I want to allow the option for users to destroy some of their audio fidelity, at
    // least one of these values must be in complete bytes.
    assert!((old % 8 == 0) || (new % 8 == 0));

    let mut scaler: u32 = 2;
    let mut ret: u32;

    if old > new {
        scaler = scaler.pow((old - new) as u32);
        ret = sample / scaler;
    } else {
        scaler = scaler.pow((new - old) as u32);
        ret = sample * scaler;
    }

    // Because I think it is funny, a user can opt to scale for a partial byte, but to keep
    // it functional, we need to scale it back up to the nearest full byte once we are done.
    // This reduces the resolution of their audio and can be useful as an artistic choice.
    if new % 8 != 0 {
        ret = scale_sample_bit_depth(ret, new, new + (8 - new % 8));
    }

    ret
}