use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

use diesel::{RunQueryDsl, SqliteConnection};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::Message, gateway::Ready},
};

use crate::{
    config::Config,
    discord::{self, roles::RoleExt},
    embed,
    models::test::Test,
};

pub struct Handler {
    pub config: Config,
    pub connection: Arc<Mutex<SqliteConnection>>,
}

impl Handler {
    fn new(config: Config, connection: SqliteConnection) -> Self {
        Self {
            config,
            connection: Arc::new(Mutex::new(connection)),
        }
    }

    fn get_pool(&self) -> Arc<Mutex<SqliteConnection>> {
        return Arc::clone(&self.connection);
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.guild_id.is_none() {
            return;
        }

        if msg.content == "!ping" {
            if let Err(why) = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.content("").set_embed(embed::build("Pong!"))
                })
                .await
            {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!database" {
            use crate::schema::test::dsl::*;

            let values = Test::new("test");

            let connection = self.get_pool();
            let handle = thread::spawn(move || {
                let mut connection = connection.lock().unwrap();
                diesel::insert_into(test)
                    .values(&values)
                    .execute(&mut *connection)
                    .expect("Error saving new test");

                let results = test
                    .first::<Test>(&mut *connection)
                    .unwrap_or_else(|_| panic!("Error loading test"));

                return results;
            });

            let results = handle.join().unwrap();
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.content("").set_embed(embed::build(&format!(
                        "Saved {} with id {}",
                        results.name,
                        results.id.unwrap_or(0)
                    )))
                })
                .await
                .unwrap();

            let connection = self.get_pool();
            thread::spawn(move || {
                let mut connection = connection.lock().unwrap();
                diesel::delete(test).execute(&mut *connection).unwrap();
            })
            .join()
            .unwrap();
        } else if msg.content == "!roles" {
            let roles = discord::roles::get_guild_roles(&ctx, msg.guild_id.unwrap()).await;

            let role_names = roles
                .iter()
                .map(|role| role.as_mention())
                .collect::<Vec<String>>()
                .join("\n");

            if let Err(why) = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.content("")
                        .set_embed(embed::build(&format!("Roles: {}", role_names)))
                })
                .await
            {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!permissions" {
            let member = msg.member(&ctx).await.unwrap();
            let permissions = discord::roles::get_permissions(&ctx, &member).await;

            let body = permissions
                .into_iter()
                .map(|(name, has_permission)| format!("{}: {}", name, has_permission))
                .collect::<Vec<String>>()
                .join("\n");

            if let Err(why) = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.content("")
                        .set_embed(embed::build(&format!("Roles:\n {}", body)))
                })
                .await
            {
                println!("Error sending message: {:?}", why);
            }
        }

        ()
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Logged in as {}!", ready.user.name);
    }
}

pub fn load_commands(config: Config, connection: SqliteConnection) -> Handler {
    Handler::new(config, connection)
}
