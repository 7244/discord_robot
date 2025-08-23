use chrono::Utc;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::{GuildId, UserId},
    },
    prelude::*,
};
use std::env;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

struct Handler {
    seen_users: Arc<RwLock<HashMap<GuildId, HashMap<UserId, i64>>>>,
}

impl Handler {
    pub async fn ext_delete_message(
        &self,
        channel_id: serenity::model::id::ChannelId,
        message_id: serenity::model::id::MessageId,
        http: &Arc<serenity::http::Http>,
    ) {
        if let Err(err) = channel_id.delete_message(http, message_id).await {
            eprintln!(
                "Failed to delete message {}:{} {:?}",
                channel_id, message_id, err
            );
        }
    }

    pub async fn ext_ban_user(
        &self,
        guild_id: serenity::model::id::GuildId,
        user_id: serenity::model::id::UserId,
        http: &Arc<serenity::http::Http>,
    ) {
        if let Err(err) = guild_id
            .ban_with_reason(http, user_id, 7, "spam or breaking server rules")
            .await
        {
            eprintln!("failed to ban user {}:{} {:?}", guild_id, user_id, err);
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut seen = self.seen_users.write().await;

        let guild_id = match msg.guild_id {
            Some(val) => val,
            None => return,
        };

        if msg.author.bot {
            return;
        }

        let message_counter = seen
            .entry(guild_id)
            .or_default()
            .entry(msg.author.id)
            .or_default();

        // user is banned. lets delete message.
        if *message_counter < 0 {
            self.ext_delete_message(msg.channel_id, msg.id, &ctx.http)
                .await;
            return;
        }

        *message_counter += 1;

        if *message_counter > 2 {
            return;
        }

        loop {
            let member = match msg.member {
                Some(val) => val,
                None => break,
            };

            let joined_at = match member.joined_at {
                Some(val) => val,
                None => break,
            };

            let since = Utc::now().signed_duration_since(joined_at.to_utc());

            if since.num_days() >= 7 {
                return;
            }

            break;
        }

        if (msg.content.to_lowercase().contains(" nigger ")
            || msg.content.to_lowercase().starts_with("nigger ")
            || msg.content.to_lowercase().ends_with(" nigger")
            || msg.content.to_lowercase() == "nigger")
            || (msg.content.to_lowercase().contains(" @everyone ")
                || msg.content.to_lowercase().starts_with("@everyone ")
                || msg.content.to_lowercase().ends_with(" @everyone")
                || msg.content.to_lowercase() == "@everyone")
            || (msg
                .content
                .to_lowercase()
                .contains("discordapp.com/invite/"))
            || (msg.content.to_lowercase().contains("t.me/+"))
            || (msg
                .author
                .name
                .to_lowercase()
                .starts_with("legitmegalinkseller"))
            || (msg.author.name.to_lowercase().starts_with("bestmegalink"))
            || (msg.content.to_lowercase().contains("message me")
                && msg.content.to_lowercase().contains("price"))
            || (msg.content.to_lowercase() == "dms open")
        {
            self.ext_ban_user(guild_id, msg.author.id, &ctx.http).await;
            self.ext_delete_message(msg.channel_id, msg.id, &ctx.http)
                .await;
            *message_counter = -1;
            return;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");

    let handler = Handler {
        seen_users: Arc::new(RwLock::new(HashMap::default())),
    };

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Err creating client");

    if let Err(err) = client.start().await {
        eprintln!("Client error: {:?}", err);
    }
}
