fn main() {
    // bytecheck can be used to validate your data if you want
    use rkyv::{Archive, CheckBytes, Deserialize, Serialize};

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
    // This will generate a PartialEq impl between our unarchived and archived types
    #[archive(compare(PartialEq))]
    // To use the safe API, you have to derive CheckBytes for the archived type
    #[archive_attr(derive(CheckBytes, Debug))]
    struct Rune {
        filename: String,
        len: u16,
        hash: [u8; 32],
    }

    let value = Rune {
        filename: "test.txt".to_string(),
        len: 42,
        hash: (0..32).collect::<Vec<u8>>().try_into().unwrap(),
    };

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
    // This will generate a PartialEq impl between our unarchived and archived types
    #[archive(compare(PartialEq))]
    // To use the safe API, you have to derive CheckBytes for the archived type
    #[archive_attr(derive(CheckBytes, Debug))]
    struct Runes {
        key: [u8; 32],
        runes: Vec<Rune>,
    };

    let values = Runes {
        key: [0; 32],
        runes: vec![value.clone(), value],
    };

    // Serializing is as easy as a single function call
    let bytes = rkyv::to_bytes::<_, 256>(&values).unwrap();
    println!("{bytes:?}");

    // Or you can customize your serialization for better performance
    // and compatibility with #![no_std] environments
    use rkyv::ser::{serializers::AllocSerializer, Serializer};

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&values).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    // You can use the safe API for fast zero-copy deserialization
    // let archived = rkyv::check_archived_root::<Test>(&bytes[..]).unwrap();
    // assert_eq!(archived, &value);

    // Or you can use the unsafe API for maximum performance
    let start = std::time::Instant::now();
    let archived = unsafe { rkyv::archived_root::<Runes>(&bytes[..]) };
    println!("{} nanos", start.elapsed().as_nanos());
    assert_eq!(archived, &values);

    // And you can always deserialize back to the original type
    let start = std::time::Instant::now();
    let deserialized: Runes = archived.deserialize(&mut rkyv::Infallible).unwrap();
    println!("{} nanos", start.elapsed().as_nanos());
    assert_eq!(deserialized, values);
}
