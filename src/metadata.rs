use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};
use symphonia::core::meta::StandardTag;
use symphonia::{
    core::{
        formats::{FormatOptions, probe::Hint},
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default::get_probe,
};
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Song {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub path: String,
    pub disc_number: u8,
    pub track_number: u8,
    pub gain: f32,
}

impl Song {
    pub fn new() -> Self {
        Self {
            title: UNKNOWN_TITLE.to_string(),
            album: UNKNOWN_ALBUM.to_string(),
            artist: UNKNOWN_ARTIST.to_string(),
            path: String::new(),
            disc_number: 1,
            track_number: 1,
            gain: 0.0,
        }
    }
}

pub fn metadata(path: impl AsRef<Path>, force_symphonia: bool) -> Result<Song, String> {
    let path = path.as_ref();
    let extension = path.extension().ok_or("Path is not audio")?;

    if extension == "flac" && !force_symphonia {
        return flac_metadata(path)
            .map_err(|err| format!("Error: ({err}) @ {}", path.to_string_lossy()));
    }

    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(format!("Error: ({err}) @ {}", path.to_string_lossy())),
    };

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut format_reader = match get_probe().probe(
        &Hint::new(),
        mss,
        FormatOptions::default()
            .prebuild_seek_index(false)
            .seek_index_fill_period_ms(20),
        MetadataOptions::default(),
    ) {
        Ok(format_reader) => format_reader,
        Err(err) => return Err(format!("Error: ({err}) @ {}", path.to_string_lossy()))?,
    };

    let mut title = String::from("Unknown Title");
    let mut album = String::from("Unknown Album");
    let mut artist = String::from("Unknown Artist");
    let mut track_number = 1;
    let mut disc_number = 1;
    let mut gain = 0.0;

    let mut metadata = format_reader.metadata();
    if let Some(latest_revision) = metadata.skip_to_latest() {
        for tag in &latest_revision.media.tags {
            if let Some(std) = &tag.std {
                match std {
                    StandardTag::AlbumArtist(tag) => artist = tag.to_string(),
                    StandardTag::Artist(tag) if artist == "Unknown Artist" => {
                        artist = tag.to_string();
                    }
                    StandardTag::Album(tag) => album = tag.to_string(),
                    StandardTag::TrackTitle(tag) => title = tag.to_string(),
                    StandardTag::TrackNumber(num) => {
                        track_number = *num as _;
                    }
                    StandardTag::DiscNumber(num) => {
                        disc_number = *num as _;
                    }
                    StandardTag::ReplayGainTrackGain(tag) => {
                        if let Some((_, tague)) = tag.split_once(' ') {
                            let db: f32 = tague.parse().unwrap_or(0.0);
                            gain = 10.0f32.powf(db / 20.0);
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    Ok(Song {
        title,
        album,
        artist,
        disc_number,
        track_number,
        path: path.to_str().ok_or("Invalid UTF-8 in path.")?.to_string(),
        gain,
    })
}

#[inline]
pub fn u24_be(reader: &mut BufReader<File>) -> u32 {
    let mut triple = [0; 4];
    reader.read_exact(&mut triple[0..3]).unwrap();
    u32::from_be_bytes(triple) >> 8
}

#[inline]
pub fn u32_le(reader: &mut BufReader<File>) -> u32 {
    let mut buffer = [0; 4];
    reader.read_exact(&mut buffer).unwrap();
    u32::from_le_bytes(buffer)
}

pub fn flac_metadata(path: impl AsRef<Path>) -> Result<Song, Box<dyn std::error::Error>> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);

    let mut flac = [0; 4];
    reader.read_exact(&mut flac)?;

    if &flac != b"fLaC" {
        Err("File is not FLAC.")?;
    }

    let mut song: Song = Song::new();
    song.path = path.as_ref().to_string_lossy().to_string();

    let mut flag = [0; 1];

    loop {
        reader.read_exact(&mut flag)?;

        // First bit of the header indicates if this is the last metadata block.
        let is_last = (flag[0] & 0x80) == 0x80;

        // The next 7 bits of the header indicates the block type.
        let block_type = flag[0] & 0x7f;
        let block_len = u24_be(&mut reader);

        //VorbisComment https://www.xiph.org/vorbis/doc/v-comment.html
        if block_type == 4 {
            let vendor_length = u32_le(&mut reader);
            reader.seek_relative(vendor_length as i64)?;

            let comment_list_length = u32_le(&mut reader);
            for _ in 0..comment_list_length {
                let length = u32_le(&mut reader) as usize;
                let mut buffer = vec![0; length as usize];
                reader.read_exact(&mut buffer)?;

                let tag = core::str::from_utf8(&buffer).unwrap();
                let (k, v) = match tag.split_once('=') {
                    Some((left, right)) => (left, right),
                    None => (tag, ""),
                };

                match k.to_ascii_lowercase().as_str() {
                    "albumartist" => song.artist = v.to_string(),
                    "artist" if song.artist == UNKNOWN_ARTIST => song.artist = v.to_string(),
                    "title" => song.title = v.to_string(),
                    "album" => song.album = v.to_string(),
                    "tracknumber" => song.track_number = v.parse().unwrap_or(1),
                    "discnumber" => song.disc_number = v.parse().unwrap_or(1),
                    "replaygain_track_gain" => {
                        //Remove the trailing " dB" from "-5.39 dB".
                        if let Some(slice) = v.get(..v.len() - 3) {
                            if let Ok(db) = slice.parse::<f32>() {
                                song.gain = 10.0f32.powf(db / 20.0);
                            }
                        }
                    }
                    _ => {}
                }
            }

            return Ok(song);
        }

        reader.seek_relative(block_len as i64)?;

        // Exit when the last header is read.
        if is_last {
            break;
        }
    }

    Err("Could not parse metadata.")?
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test() {
        const PATH: &str = "D:\\OneDrive\\Music";

        let paths: Vec<winwalk::DirEntry> = winwalk::walkdir(PATH, 0)
            .into_iter()
            .flatten()
            .filter(|entry| match entry.extension() {
                Some(ex) => {
                    matches!(ex.to_str(), Some("flac"))
                }
                None => false,
            })
            .collect();

        let songs: Vec<Result<Song, String>> = paths
            .iter()
            .map(|file| {
                flac_metadata(&file.path)
                    .map_err(|err| format!("Error: ({err}) @ {}", file.path.to_string()))
            })
            .collect();

        dbg!(&songs[0].as_ref().unwrap());
    }
}
