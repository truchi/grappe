use super::*;
use std::str::from_utf8;

test_files!("../../../../");

const EOLS: &[Eol] = &[
    Eol::CRLF,
    Eol::CR,
    Eol::LF,
    Eol::VT,
    Eol::FF,
    Eol::NEL,
    Eol::LS,
    Eol::PS,
];

#[test]
fn utf8() {
    const CONTAINS_S0: char = '→'; // [S0, 134, 146]
    const CONTAINS_S1: char = '开'; // [229, 188, S1]
    const CONTAINS_S0S1: char = '‼'; // [S0, S1, 188]
    const CONTAINS_NEL0: char = '§'; // [NEL0, 167]

    test_utf8("");
    test_utf8(&format!(
        "{}{}{}{}",
        CONTAINS_NEL0, CONTAINS_S0, CONTAINS_S1, CONTAINS_S0S1
    ));

    for file in FILES {
        test_utf8(file);
    }
}

#[test]
fn splits() {
    test_splits(&[]);
    test_splits(&[
        Split::Eol(Eol::LF),
        Split::Eol(Eol::VT),
        Split::Eol(Eol::FF),
        Split::Eol(Eol::CR),
        Split::Eol(Eol::NEL),
        Split::Eol(Eol::LS),
        Split::Eol(Eol::PS),
        Split::Eol(Eol::LF),
        Split::Eol(Eol::CR),
        Split::Eol(Eol::CR),
        Split::Eol(Eol::CR),
        Split::Eol(Eol::CRLF),
        Split::Eol(Eol::LF),
    ]);
    test_splits(&[Split::Bytes(b"Hello")]);
    test_splits(&[
        Split::Bytes(b"a"),
        Split::Eol(Eol::LF),
        Split::Bytes(b"b"),
        Split::Eol(Eol::VT),
        Split::Bytes(b"c"),
        Split::Eol(Eol::FF),
        Split::Bytes(b"d"),
        Split::Eol(Eol::CR),
        Split::Bytes(b"e"),
        Split::Eol(Eol::NEL),
        Split::Bytes(b"f"),
        Split::Eol(Eol::LS),
        Split::Bytes(b"g"),
        Split::Eol(Eol::PS),
        Split::Bytes(b"h"),
        Split::Eol(Eol::CRLF),
    ]);

    let bytes: &[&[u8]] = &[
        &[NEL0],
        &[NEL1],
        &[S0],
        &[S1],
        &[S0, S1],
        &[S0, LS2],
        &[S0, PS2],
        &[S1, LS2],
        &[S1, PS2],
        &[NEL0, S0, NEL1, S0, S1, NEL1, NEL0, LS2, NEL1, PS2],
    ];

    for &eol in EOLS {
        test_splits(&[Split::Eol(eol)]);
        test_splits(&[Split::Bytes(b"a"), Split::Eol(eol), Split::Bytes(b"b")]);
    }

    for bytes in bytes {
        test_splits(&[Split::Bytes(bytes)]);

        for &eol in EOLS {
            test_splits(&[Split::Bytes(bytes), Split::Eol(eol)]);
        }
    }
}

fn test_utf8(str: &str) {
    use std::collections::HashMap;

    fn count_eols(str: &str) -> HashMap<&[u8], (usize, usize)> {
        let mut eols = HashMap::<&[u8], (usize, usize)>::default();

        for eol in EOLS {
            eols.insert(eol.as_bytes(), (str.matches(eol.as_str()).count(), 0));
        }

        let crlfs = eols.get(Eol::CRLF.as_bytes()).unwrap().0;
        eols.get_mut(Eol::CR.as_bytes()).unwrap().1 -= crlfs;
        eols.get_mut(Eol::LF.as_bytes()).unwrap().1 -= crlfs;

        eols
    }

    fn test_or_push(eols: &mut HashMap<&[u8], (usize, usize)>, split: Split) {
        match split {
            Split::Bytes(bytes) => debug_assert!(from_utf8(bytes).is_ok()),
            Split::Eol(eol) => eols.get_mut(eol.as_bytes()).unwrap().1 += 1,
        }
    }

    let mut eols = count_eols(str);
    let mut splitter = Splitter::default();

    for split in splitter.split(str.as_bytes()) {
        if let Split::Bytes(bytes) = split {
            debug_assert!(from_utf8(bytes).is_ok());
        }

        test_or_push(&mut eols, split);
    }

    if let Some(split) = splitter.done() {
        test_or_push(&mut eols, split);
    }

    dbg!(&eols);
    for (expected, actual) in eols.values() {
        debug_assert!(expected == actual);
    }
}

fn test_splits(splits: &[Split]) {
    fn push(text: &mut Vec<u8>, eols: &mut Vec<Eol>, split: Split) {
        match split {
            Split::Bytes(b) => text.extend_from_slice(b),
            Split::Eol(eol) => {
                text.extend_from_slice(eol.as_bytes());
                eols.push(eol);
            }
        }
    }

    let mut expected_text = Vec::<u8>::new();
    let mut expected_eols = Vec::<Eol>::new();

    // The underlying byte stream
    for split in splits {
        push(&mut expected_text, &mut expected_eols, *split);
    }

    // Rebuilding the stream from splits of chunks,
    // with chunk sizes up to the full len of the stream
    for size in 1..=expected_text.len() {
        let mut splitter = Splitter::default();
        let mut actual_text = Vec::<u8>::new();
        let mut actual_eols = Vec::<Eol>::new();

        for chunk in expected_text.chunks(size) {
            for split in splitter.split(chunk) {
                dbg!(split);
                push(&mut actual_text, &mut actual_eols, split);
            }
        }

        if let Some(split) = splitter.done() {
            push(&mut actual_text, &mut actual_eols, split);
        }

        dbg!(size);
        dbg!((&expected_text, &actual_text));
        dbg!((&expected_eols, &actual_eols));
        assert!(expected_text == actual_text);
        assert!(expected_eols == actual_eols);
    }
}
