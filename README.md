# [strecken.info](http://strecken.info) telegram bot
A telegram bot that sends you strecken.info disruption updates.

## How does this bot works?

Open a chat with the bot: [t.me/strecken_info_bot](https://t.me/strecken_info_bot) and write `/start`. Now you are registered for disruption updates. The bot will fetch all disruptions from [strecken.info](http://strecken.info) and filter them and notify you if there are new or updated disruptions. If you want to unsubscribe from disruption updates just write `/unsubscribe`.

### Filters

I implemented a filter enum because I want to let the user configure these filters but I didn't implement the feature yet. I only configured some default filters: The disruption must be arbitrary and the priority must be below 30.

## Host it yourself

You are able to host this bot yourself by using docker or build a binary with `cargo` yourself.

**Docker**: ``docker run --name strecken-info-telegram -d -v database:/database -e TELEGRAM_BOT_TOKEN=YOUR_TELEGRAM_BOT_TOKEN ghcr.io/adridevelopsthings/strecken-info-telegram:main``

**By source**: Build a binary with ``cargo build --release`` and run the binary: You have to set the environment variable ``TELEGRAM_BOT_TOKEN`` and ``SQLITE_PATH``.

It's also possible to change the value of the `FETCH_EVERY_SECONDS` environment variable to an other value as 120.

## Feedback and Contribution

If you noticed some bugs or are interested in new features just create an issue. You can also contribute to this repository by forking and creating a pull request. 