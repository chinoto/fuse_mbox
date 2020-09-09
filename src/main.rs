fn main() -> std::io::Result<()> {
    let mbox_path = std::env::args_os().nth(1).expect("Mbox not provided");
    use std::fs::File;
    let file = unsafe { memmap::Mmap::map(&File::open(mbox_path)?)? };
    if let Some(window) = file.get(0..FROM.len()) {
        if window != FROM {
            panic!(
                "Invalid file; Starts with \"{}\" instead of \"From \"",
                std::str::from_utf8(&window).unwrap()
            );
        }
    } else {
        panic!("File too short")
    }
    get_email_ranges(file)
        .into_iter()
        .for_each(|(range, hash)| {
            println!("{:?} {}", range, hash);
        });
    Ok(())
}
const FROM: &[u8; 5] = b"From ";

fn get_email_ranges(file: memmap::Mmap) -> Vec<(std::ops::Range<usize>, u64)> {
    let mut ranges = vec![];
    let mut start: usize = file.iter().position(|&chr| chr == b'\n').unwrap() + 1;

    while let Some(pos) = file[start..]
        .iter()
        .enumerate()
        .find_map(|(mut pos, &chr)| {
            pos += start + 1;
            let test = chr == b'\n' && file.get(pos..)?.starts_with(FROM);
            Some(pos).filter(|_| test)
        })
    {
        ranges.push(start..pos);
        start = pos + file[pos..].iter().position(|&chr| chr == b'\n').unwrap() + 1;
    }
    ranges.push(start..file.len());

    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    ranges
        .into_iter()
        .map(|range| {
            let mut hasher: DefaultHasher = DefaultHasher::new();
            hasher.write(&file[range.clone()]);
            (range, hasher.finish())
        })
        .collect::<Vec<_>>()
}
