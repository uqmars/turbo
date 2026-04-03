# turbo
UQ MARS Discord Bot

## Dependencies 
- [Serenity](https://docs.rs/serenity/latest/serenity/)
- [Poise](https://docs.rs/poise/latest/poise/)

## Setup
While it is expected that the bot will already be set up within the UQ MARS server, in the event it needs re-installing, see the following.

In order to get the bot working, you will need to set up a Discord application, and add it to the relevant server. It will need the following configuration:
- Scopes:
    - bot
    - applications.commands
- Permissions:
    - Administrator
- Intents:
    - All

Additionally, the following environment variables should be set:
- DISCORD_TOKEN (The Discord application's bot token)
- GUILD_ID (Only needed in testing / non-release, the server ID)
