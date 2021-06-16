pub type Recipients = Vec<usize>;

pub fn single_recipient(player_number: usize) -> Recipients {
    vec![player_number]
}

pub fn everyone_but(player_count: usize, player_number: usize) -> Recipients {
    let mut v = everyone(player_count);
    v.remove(player_number);

    v
}

pub fn everyone(player_count: usize) -> Recipients {
    (0..player_count).collect::<Vec<usize>>()
}
