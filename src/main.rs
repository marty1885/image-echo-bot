
use std::env;
use std::collections::HashSet;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

pub struct ImageList {
    pub images: Vec<String>,
    pub listening: bool,
}

pub struct ImageListMap {
    pub map: HashMap<String, ImageList>,
}

impl serenity::prelude::TypeMapKey for ImageList {
    type Value = ImageList;
}

impl serenity::prelude::TypeMapKey for ImageListMap {
    type Value = ImageListMap;
}

impl ImageList {
    pub fn new() -> Self {
        ImageList {
            images: Vec::new(),
            listening: false,
        }
    }
}

impl ImageListMap {
    pub fn new() -> Self {
        ImageListMap {
            map: HashMap::new(),
        }
    }
}

lazy_static! {
    static ref IMGAE_EXTENSION: HashSet<&'static str> = {
        let mut m = HashSet::new();
        m.insert("jpg");
        m.insert("jpeg");
        m.insert("png");
        m.insert("gif");
        m.insert("webp");
        m.insert("bmp");
        m.insert("tiff");
        m
    };
}

static MAX_IMAGE_PER_SECTION: usize = 500;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
            return;
        }

        let key = msg.channel_id.to_string() + "|" + msg.author.id.to_string().as_str();
        let mut data = ctx.data.write().await;
        let image_list_map = data.get_mut::<ImageListMap>().unwrap();
        let mut image_list = image_list_map.map.entry(key.clone()).or_insert(ImageList::new());

        if msg.content == "!begin" {
            if image_list.listening {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Already listening").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }
            if let Err(why) = msg.channel_id.say(&ctx.http, "Please send images now").await {
                println!("Error sending message: {:?}", why);
            }
            image_list.listening = true;
        }
        else if msg.content == "!end" {
            if !image_list.listening {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Wasn't listening anyway").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }

            image_list.listening = false;
            let mut all_images = String::new();
            all_images += format!("Total {} images", image_list.images.len()).as_str();
            all_images += "```";
            for image in image_list.images.iter() {
                all_images += &image;
            }
            all_images += "```";
            if let Err(why) = msg.channel_id.say(&ctx.http, all_images).await {
                println!("Error sending message: {:?}", why);
            }

            image_list.images.clear();
            image_list_map.map.remove(&key);
        }
        else if image_list.listening {
            if image_list.images.len() > MAX_IMAGE_PER_SECTION {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Reached max stored images. please issue !end to end current session").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }

            // if the message is an image, add it to the list
            for attachment in msg.attachments {
                let extension = attachment.filename.split('.').last().unwrap();
                if IMGAE_EXTENSION.contains(extension) {
                    image_list.images.push(attachment.url);
                }
            }
        }


    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");
    client.data.write().await.insert::<ImageListMap>(ImageListMap::new());

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}