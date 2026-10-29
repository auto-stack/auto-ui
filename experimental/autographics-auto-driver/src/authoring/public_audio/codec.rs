//! Container and expansive codebook admission precedes codec construction.
//! All bounds are bytes in the caller's existing CPU reservation, not a cache.
use super::mixer::{Pcm, Stereo};
use std::{io::Cursor, sync::Arc};
const RAW: usize = 8 * 1024 * 1024;
const PCM: usize = 16 * 1024 * 1024;
#[derive(Debug)]
pub struct Header {
    pub rate: u32,
    pub channels: u16,
    pub frames: usize,
    pub max_frames: usize,
    pub scratch: usize,
    pub first_audio_page_is_last: bool,
    pub max_window: usize,
}
fn invalid(message: &str) -> String {
    format!("audio_invalid: {message}")
}
fn limit(message: &str) -> String {
    format!("audio_resource_limit: {message}")
}
fn u16le(bytes: &[u8], offset: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or_else(|| invalid("u16 span"))?
            .try_into()
            .unwrap(),
    ))
}
fn u32le(bytes: &[u8], offset: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| invalid("u32 span"))?
            .try_into()
            .unwrap(),
    ))
}
fn domain(rate: u32, channels: u16, frames: usize) -> Result<(), String> {
    if ![22050, 44100, 48000].contains(&rate) || !(1..=2).contains(&channels) {
        return Err("audio_unsupported: channels/sample rate".into());
    }
    if frames == 0 {
        return Err(invalid("empty PCM"));
    }
    if frames > rate as usize * 10 || frames.checked_mul(8).is_none_or(|b| b > PCM) {
        return Err(limit("duration/PCM"));
    }
    Ok(())
}
fn reserve(amount: usize, available: usize) -> Result<(), String> {
    if amount > available {
        Err(limit("shared CPU decode reservation"))
    } else {
        Ok(())
    }
}

fn wav(bytes: &[u8]) -> Result<(Header, std::ops::Range<usize>), String> {
    if bytes.get(..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err("audio_unsupported: container".into());
    }
    if u32le(bytes, 4)? as usize + 8 != bytes.len() {
        return Err(invalid("RIFF length"));
    }
    let mut cursor = 12;
    let mut format = None;
    let mut data = None;
    while cursor < bytes.len() {
        let len = u32le(bytes, cursor + 4)? as usize;
        let start = cursor + 8;
        let end = start
            .checked_add(len)
            .ok_or_else(|| invalid("chunk overflow"))?;
        let body = bytes.get(start..end).ok_or_else(|| invalid("chunk span"))?;
        match bytes.get(cursor..cursor + 4) {
            Some(b"fmt ") => {
                if format.is_some() {
                    return Err(invalid("duplicate fmt"));
                }
                if len < 16 {
                    return Err(invalid("fmt length"));
                }
                if u16le(body, 0)? != 1 || u16le(body, 14)? != 16 {
                    return Err("audio_unsupported: PCM16 WAV required".into());
                }
                let channels = u16le(body, 2)?;
                let rate = u32le(body, 4)?;
                domain(rate, channels, 1)?;
                if u16le(body, 12)? != channels * 2
                    || u32le(body, 8)?
                        != rate
                            .checked_mul(u32::from(channels) * 2)
                            .ok_or_else(|| invalid("byte rate overflow"))?
                {
                    return Err(invalid("PCM alignment/byte rate"));
                }
                format = Some((rate, channels));
            }
            Some(b"data") => {
                if data.replace(start..end).is_some() {
                    return Err(invalid("duplicate data"));
                }
            }
            _ => {}
        }
        cursor = end
            .checked_add(len % 2)
            .ok_or_else(|| invalid("chunk pad"))?;
        if cursor > bytes.len() {
            return Err(invalid("missing RIFF padding"));
        }
    }
    let (rate, channels) = format.ok_or_else(|| invalid("missing fmt"))?;
    if !(1..=2).contains(&channels) {
        return Err("audio_unsupported: channels".into());
    }
    let data = data.ok_or_else(|| invalid("missing data"))?;
    if data.len() % (channels as usize * 2) != 0 {
        return Err(invalid("partial PCM frame"));
    }
    let frames = data.len() / (channels as usize * 2);
    domain(rate, channels, frames)?;
    Ok((
        Header {
            rate,
            channels,
            frames,
            max_frames: frames,
            scratch: 0,
            first_audio_page_is_last: false,
            max_window: 0,
        },
        data,
    ))
}

struct Bits<'a> {
    data: &'a [u8],
    bit: usize,
}
impl Bits<'_> {
    fn read(&mut self, n: usize) -> Result<u32, String> {
        if n > 32
            || self
                .bit
                .checked_add(n)
                .is_none_or(|end| end > self.data.len() * 8)
        {
            return Err(invalid("Vorbis setup bit span"));
        }
        let mut value = 0;
        for i in 0..n {
            value |= u32::from((self.data[self.bit / 8] >> (self.bit % 8)) & 1) << i;
            self.bit += 1;
        }
        Ok(value)
    }
    fn skip(&mut self, n: usize) -> Result<(), String> {
        let end = self
            .bit
            .checked_add(n)
            .ok_or_else(|| invalid("setup overflow"))?;
        if end > self.data.len() * 8 {
            return Err(invalid("setup span"));
        }
        self.bit = end;
        Ok(())
    }
}
fn codebook_reservation(packet: &[u8], available: usize) -> Result<usize, String> {
    if packet.get(..7) != Some(b"\x05vorbis") {
        return Err(invalid("setup signature"));
    }
    let mut bits = Bits {
        data: packet,
        bit: 56,
    };
    let count = bits.read(8)? as usize + 1;
    // Fixed conservative envelope for floor/residue/mapping/window/MDCT state
    // in the pinned decoder (<=64 each, stereo, block exponent <=13), plus
    // decoder/packet/audio-block temporary storage. VQ/VLC are added below.
    let mut memory = 12 * 1024 * 1024;
    reserve(memory, available)?;
    for _ in 0..count {
        if bits.read(24)? != 0x564342 {
            return Err(invalid("codebook sync"));
        }
        let dimensions = bits.read(16)?;
        let entries = bits.read(24)?;
        if dimensions == 0 || entries == 0 {
            return Err(invalid("empty codebook"));
        }
        // Reserve bounded entry arrays/tree values before scanning lengths.
        // Then count the possible occupied prefixes at8/16/24bits. Treating
        // every entry as three disjoint256-entry blocks falsely rejects normal
        // Vorbis sources; prefix cardinality bounds the real builder tightly.
        memory = memory
            .checked_add(entries as usize * 128)
            .ok_or_else(|| limit("codebook multiplication"))?;
        reserve(memory, available)?;
        let mut long = [0usize; 3];
        let mut count_length = |length: u32, n: u32| {
            for (level, total) in long.iter_mut().enumerate() {
                if length > 8 * (level as u32 + 1) {
                    *total += n as usize
                }
            }
        };
        if bits.read(1)? == 0 {
            let sparse = bits.read(1)? != 0;
            for _ in 0..entries {
                if !sparse || bits.read(1)? != 0 {
                    count_length(bits.read(5)? + 1, 1);
                }
            }
        } else {
            let mut length = bits.read(5)? + 1;
            let mut used = 0;
            while used < entries {
                if length > 32 {
                    return Err(invalid("ordered codeword length"));
                }
                let n = bits.read((32 - (entries - used).leading_zeros()) as usize)?;
                used = used
                    .checked_add(n)
                    .ok_or_else(|| invalid("ordered entries"))?;
                if used > entries {
                    return Err(invalid("ordered entries"));
                }
                count_length(length, n);
                length += 1;
            }
        }
        let blocks = 1 + long[0].min(256) + long[1].min(65536) + long[2].min(16777216);
        // 2x Vec table capacity (256*8), block structs, parent BTree nodes,
        // breadth-first queue and allocation headers, rounded upward.
        memory = memory
            .checked_add(blocks * 4608)
            .ok_or_else(|| limit("VLC blocks"))?;
        reserve(memory, available)?;
        let lookup = bits.read(4)?;
        if lookup > 2 {
            return Err(invalid("codebook lookup type"));
        }
        if lookup != 0 {
            memory = memory
                .checked_add(entries as usize * dimensions as usize * 8)
                .ok_or_else(|| limit("VQ multiplication"))?;
            reserve(memory, available)?;
            bits.skip(64)?;
            let width = bits.read(4)? as usize + 1;
            bits.read(1)?;
            let values = if lookup == 2 {
                entries as usize * dimensions as usize
            } else {
                let mut root = 1u32;
                while (root + 1)
                    .checked_pow(dimensions)
                    .is_some_and(|p| p <= entries)
                {
                    root += 1;
                }
                // Symphonia0.5.5 uses f32 root + u32 pow assertions. Refuse a
                // setup that its arithmetic cannot represent before construction.
                if (root + 1).checked_pow(dimensions).is_none() {
                    return Err("audio_unsupported: decoder codebook arithmetic".into());
                }
                root as usize
            };
            bits.skip(
                values
                    .checked_mul(width)
                    .ok_or_else(|| limit("lookup bits"))?,
            )?;
        }
    }
    Ok(memory)
}
fn crc(page: &[u8]) -> u32 {
    let mut sum = 0u32;
    for (i, byte) in page.iter().enumerate() {
        sum ^= u32::from(if (22..26).contains(&i) { 0 } else { *byte }) << 24;
        for _ in 0..8 {
            sum = if sum & 0x80000000 != 0 {
                (sum << 1) ^ 0x04c11db7
            } else {
                sum << 1
            };
        }
    }
    sum
}
fn ogg(bytes: &[u8], available: usize) -> Result<Header, String> {
    let mut offset = 0;
    let mut seq = 0u32;
    let mut serial = None;
    let mut packet = Vec::new();
    let mut packets = 0;
    let mut rate = 0;
    let mut channels = 0;
    let mut frames = 0;
    let mut eos = false;
    let mut scratch = 0;
    let mut first_audio_page_is_last = false;
    let mut max_window = 0;
    reserve(bytes.len() * 8 + 12 * 1024 * 1024, available)?;
    while offset < bytes.len() {
        if eos {
            return Err("audio_unsupported: chained/trailing Ogg stream".into());
        }
        let head = bytes
            .get(offset..offset + 27)
            .ok_or_else(|| invalid("Ogg page header"))?;
        if &head[..4] != b"OggS" || head[4] != 0 {
            return Err(invalid("Ogg signature/version"));
        }
        let flags = head[5];
        let segments = head[26] as usize;
        if flags & !7 != 0 || (flags & 1 != 0) != !packet.is_empty() {
            return Err(invalid("Ogg continuation flags"));
        }
        if (offset == 0) != (flags & 2 != 0) {
            return Err(invalid("Ogg BOS"));
        }
        let table = bytes
            .get(offset + 27..offset + 27 + segments)
            .ok_or_else(|| invalid("Ogg lacing table"))?;
        let end = offset + 27 + segments + table.iter().map(|n| *n as usize).sum::<usize>();
        let page = bytes
            .get(offset..end)
            .ok_or_else(|| invalid("Ogg page span"))?;
        if u32le(head, 18)? != seq || serial.is_some_and(|s| s != u32le(head, 14).unwrap()) {
            return Err(invalid("Ogg serial/sequence"));
        }
        serial = Some(u32le(head, 14)?);
        seq = seq
            .checked_add(1)
            .ok_or_else(|| invalid("Ogg sequence exhaustion"))?;
        if crc(page) != u32le(head, 22)? {
            return Err(invalid("Ogg CRC"));
        }
        let granule = u64::from_le_bytes(head[6..14].try_into().unwrap());
        if granule != u64::MAX {
            if granule < frames as u64 {
                return Err(invalid("Ogg nonmonotonic granule"));
            }
            if granule > 48000 * 10 {
                return Err(limit("Ogg granule/duration"));
            }
            frames = granule as usize;
        }
        let mut body = offset + 27 + segments;
        for &size in table {
            let n = size as usize;
            if packet.len() + n > RAW {
                return Err(limit("Ogg packet"));
            }
            packet.extend_from_slice(&bytes[body..body + n]);
            body += n;
            if size < 255 {
                match packets {
                    0 => {
                        if packet.len() != 30
                            || packet.get(..7) != Some(b"\x01vorbis")
                            || u32le(&packet, 7)? != 0
                        {
                            return Err("audio_unsupported: Ogg/Vorbis identification".into());
                        }
                        channels = u16::from(packet[11]);
                        rate = u32le(&packet, 12)?;
                        domain(rate, channels, 1)?;
                        let short = packet[28] & 15;
                        let long = packet[28] >> 4;
                        max_window = (1usize << long) / 2;
                        if !(6..=13).contains(&short)
                            || !(short..=13).contains(&long)
                            || packet[29] != 1
                        {
                            return Err(invalid("Vorbis blocksize/framing"));
                        }
                    }
                    1 => {
                        if packet.get(..7) != Some(b"\x03vorbis") {
                            return Err(invalid("comment signature"));
                        }
                    }
                    2 => {
                        scratch = codebook_reservation(&packet, available - bytes.len() * 8)?;
                    }
                    _ => {
                        if packets == 3 {
                            first_audio_page_is_last = flags & 4 != 0;
                        }
                        if packet.is_empty() || packet[0] & 1 != 0 {
                            return Err(invalid("Vorbis audio packet"));
                        }
                    }
                }
                packets += 1;
                packet.clear();
            }
        }
        eos = flags & 4 != 0;
        if eos && granule == u64::MAX {
            return Err(invalid("EOS granule"));
        }
        offset = end;
    }
    if !eos || !packet.is_empty() || packets < 4 {
        return Err(invalid("incomplete Ogg"));
    }
    domain(rate, channels, frames)?;
    // Reserve bounded leading/trailing codec windows before decoder construction.
    // Final PCM follows the container timeline, never raw padded block length.
    let max_frames = frames + 2 * max_window;
    reserve(bytes.len() * 8 + max_frames * 16 + scratch, available)?;
    Ok(Header {
        rate,
        channels,
        frames,
        max_frames,
        scratch,
        first_audio_page_is_last,
        max_window,
    })
}
pub fn inspect(bytes: &[u8], available: usize) -> Result<Header, String> {
    if bytes.is_empty() || bytes.len() > RAW {
        return Err(limit("raw size"));
    }
    if bytes.starts_with(b"OggS") {
        ogg(bytes, available)
    } else {
        let (header, _) = wav(bytes)?;
        reserve(bytes.len() + header.frames * 16, available)?;
        Ok(header)
    }
}
pub fn decode(bytes: Arc<[u8]>, available: usize) -> Result<(Arc<Pcm>, Header), String> {
    let mut header = inspect(&bytes, available)?;
    super::super::assets::record_decode();
    let mut frames = Vec::<Stereo>::with_capacity(header.max_frames);
    if bytes.starts_with(b"RIFF") {
        let (_, data) = wav(&bytes)?;
        for frame in bytes[data].chunks_exact(header.channels as usize * 2) {
            let left = f32::from(i16::from_le_bytes(frame[..2].try_into().unwrap())) / 32768.0;
            let right = if header.channels == 2 {
                f32::from(i16::from_le_bytes(frame[2..4].try_into().unwrap())) / 32768.0
            } else {
                left
            };
            frames.push([left, right]);
        }
    } else {
        use symphonia::core::{
            audio::{AudioBufferRef, Signal},
            codecs::DecoderOptions,
            formats::FormatOptions,
            io::MediaSourceStream,
            meta::MetadataOptions,
            probe::Hint,
        };
        let stream = MediaSourceStream::new(Box::new(Cursor::new(bytes)), Default::default());
        let mut format = symphonia::default::get_probe()
            .format(
                &Hint::new(),
                stream,
                &FormatOptions {
                    enable_gapless: false,
                    ..Default::default()
                },
                &MetadataOptions::default(),
            )
            .map_err(|e| invalid(&e.to_string()))?
            .format;
        let track = format
            .default_track()
            .ok_or_else(|| invalid("Vorbis track"))?;
        if track.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_VORBIS {
            return Err("audio_unsupported: Vorbis codec required".into());
        }
        // The generic Ogg demuxer treats tail padding on a first-and-last
        // audio page as start delay. Decode raw blocks and apply the bounded
        // container interval explicitly. On multi-page streams its start bound
        // still supplies the legitimate leading delay/positive timestamp.
        let start_trim = if header.first_audio_page_is_last {
            0
        } else {
            track.codec_params.delay.unwrap_or(0) as usize
        };
        let start_ts = usize::try_from(track.codec_params.start_ts)
            .map_err(|_| invalid("Vorbis start timestamp"))?;
        if start_trim > header.max_window || start_ts >= header.frames {
            return Err(invalid("Vorbis leading interval"));
        }
        let valid_frames = header.frames - start_ts;
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| invalid(&e.to_string()))?;
        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(symphonia::core::errors::Error::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break
                }
                Err(e) => return Err(invalid(&e.to_string())),
            };
            let samples = decoder
                .decode(&packet)
                .map_err(|e| invalid(&e.to_string()))?;
            let AudioBufferRef::F32(audio) = samples else {
                return Err(invalid("Vorbis float PCM"));
            };
            if audio.frames() > 8192
                || audio.spec().rate != header.rate
                || audio.spec().channels.count() != header.channels as usize
            {
                return Err(invalid("decoded block domain"));
            }
            if frames.len() + audio.frames() > header.max_frames {
                return Err(limit("decoded frames exceed bounded prediction/duration"));
            }
            for i in 0..audio.frames() {
                let left = audio.chan(0)[i];
                let right = audio.chan(if header.channels == 1 { 0 } else { 1 })[i];
                if !left.is_finite() || !right.is_finite() {
                    return Err(invalid("nonfinite decoded PCM"));
                }
                frames.push([left, right]);
            }
        }
        let end = start_trim
            .checked_add(valid_frames)
            .ok_or_else(|| invalid("Vorbis interval overflow"))?;
        if frames.len() < end || frames.len() - end > header.max_window {
            return Err(invalid("Vorbis granule/decoded padding mismatch"));
        }
        frames.truncate(end);
        if start_trim > 0 {
            frames.drain(..start_trim);
        }
        domain(header.rate, header.channels, frames.len())?;
        header.frames = frames.len();
    }
    Ok((
        Arc::new(Pcm {
            rate: header.rate,
            frames: frames.into(),
        }),
        header,
    ))
}
