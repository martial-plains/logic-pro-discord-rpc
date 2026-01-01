use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, ActivityType, Assets},
};

#[derive(Debug)]
pub struct DiscordIpcClientWrapper {
    client: DiscordIpcClient,
}

impl DiscordIpcClientWrapper {
    pub fn new() -> Option<Self> {
        let client_id = std::env::var("CLIENT_ID").expect("Missing client id");
        let mut client = DiscordIpcClient::new(&client_id);
        if client.connect().is_ok() {
            println!("Connected to Discord IPC.");
            Some(Self { client })
        } else {
            eprintln!("Failed to connect to Discord IPC.");
            None
        }
    }

    pub fn clear_activity(&mut self) {
        let _ = self.client.clear_activity();
    }

    pub fn update_state(&mut self, state: &str) {
        let activity = Activity::new()
            .activity_type(ActivityType::Playing)
            .assets(Assets::new().large_image("logic-pro"))
            .state(state);
        let _ = self.client.set_activity(activity);
    }
}
