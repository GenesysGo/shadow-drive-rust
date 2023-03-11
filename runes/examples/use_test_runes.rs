use runes::{inscribe_runes, Rune, Runes};

inscribe_runes!("./aidan.runes");

fn main() {
    let expected = Runes {
        storage_account: [
            1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
            14, 14, 15, 15, 16, 16,
        ],
        runes: vec![Rune {
            name: "Aidan Tooty".to_string(),
            len: 128,
            hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                10, 11, 12, 13, 14, 15, 16,
            ],
        }],
    };

    let runes = unsafe { get_runes_unchecked() };
    assert_eq!(runes, &expected);
    let rune = runes.get_rune("Aidan Tooty").unwrap();
    println!("{rune:?}");
}
