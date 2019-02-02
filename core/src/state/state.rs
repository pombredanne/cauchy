
pub struct State {
	utxo_id_set: Arc<Mutex<(Bytes, u32)>>,
	sketch: Arc<Mutex<u64>>,
}
