

macro_rules! command_peer_else_add {
    ($arena: ident, $pk: ident, $fn_name: ident, $input: ident) => {
        let arena_r = $arena.read().unwrap();
        match arena_r.get_peer(&$pk) {
            Some(peer_status) => peer_status.$fn_name($input),
            None => {
                drop(arena_r);
                let mut arena_w = $arena.write().unwrap();
                arena_w.new_peer(&$pk);
                drop(arena_w);

                let arena_r = $arena.read().unwrap();
                arena_r
                    .get_peer(&$pk)
                    .unwrap()
                    .$fn_name($input);
            }
        }
    };
}