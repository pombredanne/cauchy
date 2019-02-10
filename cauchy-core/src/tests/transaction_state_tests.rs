mod spending {
    use state::spend_state::*;

    #[test]
    fn test_spend() {
        let mut tx_state = SpendState::init(5);
        match tx_state.spend(3) {
            Ok(()) => assert!(true),
            Err(_err) => assert!(false),
        }
    }

    #[test]
    fn test_doublespend() {
        let mut tx_state = SpendState::init(5);
        tx_state.spend(3).unwrap();
        match tx_state.spend(3) {
            Ok(()) => assert!(false),
            Err(_err) => assert!(true),
        }
    }

    #[test]
    fn test_doubleunspend() {
        let mut tx_state = SpendState::init(5);
        tx_state.spend(3).unwrap();
        tx_state.unspend(3).unwrap();
        match tx_state.unspend(3) {
            Ok(()) => assert!(false),
            Err(_err) => assert!(true),
        }
    }
}

mod serialisation {
    use bytes::Bytes;
    use state::spend_state::*;

    #[test]
    fn serialise_deserialise() {
        let mut tx_state_before = SpendState::init(5);
        tx_state_before.spend(3).unwrap();
        let raw = Bytes::from(tx_state_before);
        let mut tx_state_after = SpendState::from(raw);
        match tx_state_after.spend(3) {
            Ok(()) => assert!(false),
            Err(_err) => assert!(true),
        }
    }
}
