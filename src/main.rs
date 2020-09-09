fn main() -> std::io::Result<()> {
    assert!(
        std::mem::size_of::<usize>() <= 8,
        "Assumption broken: usize greater than 8 bytes"
    );
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
    get_email_ranges(&file)
        .into_iter()
        .for_each(|(range, _hash)| {
            println!("{}", u64_to_hex(range.start as _, &mut Default::default()));
        });
    Ok(())
}
const FROM: &[u8; 5] = b"From ";

fn u64_to_hex(u: u64, hex: &mut [u8; 8 * 2]) -> &str {
    let mut bytes: [u8; 8] = Default::default();
    for (i, elem) in bytes.iter_mut().rev().enumerate() {
        *elem = ((u >> (i * 8)) & 255) as u8;
    }
    hex::encode_to_slice(&bytes, hex).expect("hex is twice as large as bytes");
    let start = hex.iter().position(|&x| x != b'0').unwrap_or(0) & !0b11;
    std::str::from_utf8(&hex[start..]).unwrap()
}

fn hash(content: &[u8]) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(content);
    hasher.finish()
}

fn get_email_ranges(file: &memmap::Mmap) -> Vec<(std::ops::Range<usize>, u64)> {
    // This could be made into an iterator with some effort...
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
        ranges.push((start..pos, hash(&file[start..pos])));
        start = pos
            + file[pos..]
                .iter()
                .position(|&chr| chr == b'\n')
                .unwrap_or(0)
            + 1;
    }
    ranges.push((start..file.len(), hash(&file[start..file.len()])));

    ranges
}
