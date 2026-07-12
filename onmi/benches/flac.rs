use criterion::{Criterion, black_box, criterion_group, criterion_main};
use miniwalk::*;
use onmi::*;

fn custom(files: &[DirEntry]) -> Vec<Result<Song, String>> {
    files
        .iter()
        .map(|file| match flac_metadata(&file.path) {
            Ok(song) => Ok(song),
            Err(err) => Err(format!("Error: ({err}) @ {}", file.path.display())),
        })
        .collect()
}

fn symphonia(files: &[DirEntry]) -> Vec<Result<Song, String>> {
    files
        .iter()
        .map(|entry| metadata(&entry.path, true))
        .collect()
}

const PATH: &str = "D:\\OneDrive\\Music";

fn flac(c: &mut Criterion) {
    let mut group = c.benchmark_group("flac");
    group.sample_size(10);

    let paths: Vec<DirEntry> = walkdir(PATH, 0)
        .into_iter()
        .flatten()
        .filter(|entry| match entry.extension() {
            Some(ex) => {
                matches!(ex.to_str(), Some("flac"))
            }
            None => false,
        })
        .collect();

    group.bench_function("custom", |b| {
        b.iter(|| {
            custom(black_box(&paths));
        });
    });

    group.bench_function("symphonia", |b| {
        b.iter(|| {
            symphonia(black_box(&paths));
        });
    });

    group.finish();
}

criterion_group!(benches, flac);
criterion_main!(benches);
