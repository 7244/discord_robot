use chrono::Utc;
use serenity::{
    all::{ShardStageUpdateEvent, Timestamp},
    async_trait,
    gateway::ConnectionStage,
    model::{
        channel::Message,
        gateway::Ready,
        id::{GuildId, UserId},
    },
    prelude::*,
};
use std::env;
use std::{collections::HashMap, sync::Arc};

struct UserIdValue {
    total_msg_count: i64,
    message_channel: [serenity::model::id::ChannelId; 3],
    message_content: [String; 3],
    message_timestamp: [Timestamp; 3],
}
impl Default for UserIdValue {
    fn default() -> Self {
        Self {
            total_msg_count: 0,
            message_channel: [serenity::model::id::ChannelId::default(); 3],
            message_content: [String::new(), String::new(), String::new()],
            message_timestamp: [Timestamp::default(); 3],
        }
    }
}

struct Handler {
    seen_users: Arc<Mutex<HashMap<GuildId, HashMap<UserId, UserIdValue>>>>,
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
        println!("called ban {}", Utc::now());
    }

    pub fn ext_warn_with_msg(&self, msg: &Message, err: &str) {
        println!(
            "WARNING: \"{}\" with time:{} guild_id:{} author_id:{} msg: \"{}\"",
            err,
            msg.timestamp.naive_utc(),
            msg.guild_id.unwrap(),
            msg.author.id,
            msg.content.chars().take(128).collect::<String>()
        );
    }
}

fn is_string_contains_word_nocase(s: &str, word: &str) -> bool {
    s.to_lowercase() == word
        || s.to_lowercase().starts_with(&format!("{} ", word))
        || s.to_lowercase().ends_with(&format!(" {}", word))
        || s.to_lowercase().contains(&format!(" {} ", word))
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut seen = self.seen_users.lock().await;

        let guild_id = match msg.guild_id {
            Some(val) => val,
            None => return,
        };

        if msg.author.bot {
            return;
        }

        let user_value = seen
            .entry(guild_id)
            .or_default()
            .entry(msg.author.id)
            .or_default();

        // user is banned. lets delete message.
        if user_value.total_msg_count < 0 {
            self.ext_delete_message(msg.channel_id, msg.id, &ctx.http)
                .await;
            return;
        }

        loop {
            let member = match &msg.member {
                Some(val) => val,
                None => {
                    self.ext_warn_with_msg(&msg, "match msg.member");
                    break;
                }
            };

            let joined_at = match &member.joined_at {
                Some(val) => val,
                None => {
                    self.ext_warn_with_msg(&msg, "match member.joined_at");
                    break;
                }
            };

            let since = Utc::now().signed_duration_since(joined_at.to_utc());

            if since.num_days() >= 7 {
                return;
            }

            break;
        }

        if user_value.total_msg_count < 3 {
            user_value.message_channel[user_value.total_msg_count as usize] = msg.channel_id;
            user_value.message_timestamp[user_value.total_msg_count as usize] = msg.timestamp;
            user_value.message_content[user_value.total_msg_count as usize] = msg.content.clone();
        }

        user_value.total_msg_count += 1;

        if user_value.total_msg_count == 3 {
            let mut all_different_channel = true;
            for i0 in 0..3 {
                for i1 in 0..3 {
                    if i1 == i0 {
                        continue;
                    }
                    if user_value.message_channel[i0] == user_value.message_channel[i1] {
                        all_different_channel = false;
                    }
                }
            }

            let mut refe = user_value.message_timestamp[0];
            let mut total = 0;
            for i in 1..3 {
                let cur = user_value.message_timestamp[i];
                total += cur.signed_duration_since(*refe).num_seconds();
                refe = cur;
            }
            total /= 2;
            if all_different_channel == true {
                if total < 180 {
                    self.ext_ban_user(guild_id, msg.author.id, &ctx.http).await;
                    self.ext_delete_message(msg.channel_id, msg.id, &ctx.http)
                        .await;
                    user_value.total_msg_count = -1;
                    return;
                }
            } else {
                let mut all_same = true;
                for i in 1..3 {
                    if user_value.message_content[0] != user_value.message_content[i] {
                        all_same = false;
                    }
                }
                if all_same == true && user_value.message_content[0].len() > 128 && total < 180 {
                    self.ext_ban_user(guild_id, msg.author.id, &ctx.http).await;
                    self.ext_delete_message(msg.channel_id, msg.id, &ctx.http)
                        .await;
                    user_value.total_msg_count = -1;
                    return;
                }
            }
        }

        if user_value.total_msg_count > 2 {
            return;
        }

        if is_string_contains_word_nocase(&msg.content, "nigger")
            || is_string_contains_word_nocase(&msg.content, "@everyone")
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
            || (msg.author.name.to_lowercase().starts_with("hotlinks"))
            || (msg.author.name.to_lowercase().starts_with("bestmegalink"))
            || (msg.author.name.to_lowercase().contains("mommy")
                && msg.author.name.to_lowercase().contains("goon"))
            || ((is_string_contains_word_nocase(&msg.content, "dm")
                || is_string_contains_word_nocase(&msg.content, "message me"))
                && (is_string_contains_word_nocase(&msg.content, "free")
                    || is_string_contains_word_nocase(&msg.content, "price")))
            || (msg.content.to_lowercase().contains("message me")
                && msg.content.to_lowercase().contains("price"))
            || (msg.content.to_lowercase() == "dms open")
        {
            self.ext_ban_user(guild_id, msg.author.id, &ctx.http).await;
            self.ext_delete_message(msg.channel_id, msg.id, &ctx.http)
                .await;
            user_value.total_msg_count = -1;
            return;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
    async fn shard_stage_update(&self, _: Context, event: ShardStageUpdateEvent) {
        match event.new {
            ConnectionStage::Connected => println!("Shard {} connected", event.shard_id),
            ConnectionStage::Connecting => println!("Shard {} connecting", event.shard_id),
            ConnectionStage::Disconnected => println!("Shard {} disconnected", event.shard_id),
            ConnectionStage::Handshake => println!("Shard {} Handshake", event.shard_id),
            ConnectionStage::Identifying => println!("Shard {} identifying", event.shard_id),
            ConnectionStage::Resuming => println!("Shard {} resuming", event.shard_id),
            _ => println!("Shard {} unknown event", event.shard_id),
        }
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");

    let handler = Handler {
        seen_users: Arc::new(Mutex::new(HashMap::default())),
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
