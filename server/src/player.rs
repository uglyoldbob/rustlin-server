pub struct Player {
    name: String,
    id: Option<u32>,
}

impl Player {
    pub fn new() -> Option<Player> {
        Some(Player {
            name: "bob".to_string(),
            id: None,
        })
    }

    pub fn valid_name(n: String) -> bool {
        !n.is_empty()
    }
}
