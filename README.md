# Twitch message logger

- Joins every channel in `data/channels` (login name on every line).
- Listens for all messages in the joined twitch channels. Adds each message to the `data/logs`.
- If a message was sent by someone who is verified, starts listening for messages on that channel too. This way the bot can spread naturally.
