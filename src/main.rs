use poise::{
    CreateReply,
    serenity_prelude::{self as serenity},
};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn unban(ctx: Context<'_>) -> Result<(), Error> {
    let is_admin = ctx
        .author_member()
        .await
        .ok_or("This command is only for guilds")?
        .permissions
        .unwrap()
        .administrator();

    if !is_admin {
        ctx.say("This command is admin only").await?;
        return Ok(());
    }

    let guild = ctx
        .partial_guild()
        .await
        .ok_or("This command is guild only")?;

    let bans = guild.bans(ctx.http(), None, None).await?;

    let status_message = ctx.say("Unban started...").await?;

    let mut count = 0;

    for user in bans {
        let result = guild.unban(ctx.http(), user.user.id).await;

        if let Err(err) = result {
            ctx.say(format!("Failed to unban user: {}: {:?}", user.user.id, err))
                .await?;
        } else {
            count += 1;
            if count % 10 == 0 {
                let updated_message = format!("Unabanned {} users so far...", count);
                status_message
                    .edit(ctx, CreateReply::default().content(updated_message))
                    .await
                    .unwrap();
            }
        }
    }

    status_message.delete(ctx).await?;
    ctx.say("Done :)").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MODERATION
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![unban()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
