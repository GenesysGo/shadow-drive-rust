use runes::{Rune, Runes};

fn main() {
    let runes = Runes {
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

    runes.save("aidan".into()).unwrap();
}
