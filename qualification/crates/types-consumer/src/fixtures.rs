use candidate_types::{Account, Principal};

pub fn check() {
    for len in 0..=29 {
        let bytes = vec![0xa5; len];
        let owner = Principal::from_slice(&bytes);
        // This assignment crosses Account's public field with the direct type.
        let account = Account::from(owner);
        let recovered: Principal = account.owner;
        assert_eq!(recovered, owner);
        assert_eq!(account.to_string().parse::<Account>().unwrap(), account);
        for subaccount in [None, Some([0; 32]), Some([255; 32])] {
            let value = Account { owner, subaccount };
            let json = serde_json::to_string(&value).unwrap();
            let decoded: Account = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded.owner, owner);
            assert_eq!(decoded.subaccount, subaccount);
        }
    }
    assert!(Principal::try_from_slice(&[0; 30]).is_err());
    assert!("aaaaa-aa-nygwkoy.2".parse::<Account>().is_err());
    assert_eq!(
        "aaaaa-aa-nygwkoy.1"
            .parse::<Account>()
            .unwrap()
            .subaccount
            .unwrap()[31],
        1
    );
    #[cfg(feature = "self-authentication")]
    self_authentication();
}

#[cfg(feature = "self-authentication")]
fn self_authentication() {
    // SHA-224 known answers independently obtained with Node/OpenSSL, not this crate.
    for (len, expected) in [
        (
            0,
            "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f02",
        ),
        (
            3,
            "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da702",
        ),
        (
            55,
            "d9d3dba3197075d144273710b8a3d7d9099111f5b4be2655af5a5ec002",
        ),
        (
            56,
            "640c1ea565519d6549b066249a715a5bddaa8b6f3901d2f084ba496102",
        ),
        (
            64,
            "df244c8f348498c2812ccb2477817855657675f20771fd50ce7ec38002",
        ),
        (
            65,
            "6f7244e7a2554e8d32baebed338263d3cc73ef80343311134ef6bc8902",
        ),
        (
            1024,
            "0fd445d40e0a0160183d188bda9bd485ddaa494f6bf9afb03ee60a5902",
        ),
    ] {
        let input = if len == 3 {
            b"abc".to_vec()
        } else {
            vec![0xa5; len]
        };
        let identity = Principal::self_authenticating(&input);
        let expected: Vec<_> = expected
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect();
        assert_eq!(identity.as_slice(), expected);
        assert_eq!(identity.len(), 29);
        assert_eq!(Principal::from_text(identity.to_text()).unwrap(), identity);
    }
}
