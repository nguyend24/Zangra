use crate::four_horseman_server::utils::add_role_rules_verified;
use crate::twitch_webhook_handler::set_up_twitch_webhooks;
use crate::VERSION;
use serenity::model::prelude::{Reaction, Ready};
use serenity::prelude::{Context, EventHandler};
use crate::test_server::{_reaction_add_test, _reaction_remove_test};
use serenity::model::channel::Message;

pub struct Handler;

impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        add_role_rules_verified(&ctx, &reaction);
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("Version: {}", VERSION);

        // set_up_twitch_webhooks(ctx);
    }
}
