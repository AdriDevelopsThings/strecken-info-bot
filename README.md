# [strecken.info](http://strecken.info) bot
A telegram/mastodon bot that sends you strecken.info disruption updates.

## [Telegram] How does this bot works?

Open a chat with the bot: [t.me/strecken_info_bot](https://t.me/strecken_info_bot) and write `/start`. Now you are registered for disruption updates. The bot will fetch all disruptions from [strecken.info](http://strecken.info) and filter them and notify you if there are new or updated disruptions. If you want to unsubscribe from disruption updates just write `/unsubscribe`.

## [Mastodon] How does this bot works?
Just follow [https://social.adridoesthings.com/@strecken_info](https://social.adridoesthings.com/@strecken_info) on mastodon.

## Host it yourself

You are able to host this bot yourself by using docker or build a binary with `cargo` yourself.

**Docker**: ``docker run --name strecken-info-telegram -d -v database:/database -e TELEGRAM_BOT_TOKEN=YOUR_TELEGRAM_BOT_TOKEN -e MASTODON_URL=https://your.mastodon.instance.com -e MASTODON_ACCESS_TOKEN=YOUR_ACCESS_TOKEN ghcr.io/adridevelopsthings/strecken-info-telegram:main``

**By source**: Build a binary with ``cargo build --release`` and run the binary: You have to set the environment variable ``SQLITE_PATH`` and ``TELEGRAM_BOT_TOKEN`` for telegram and/or `MASTODON_URL` and `MASTODON_ACCESS_TOKEN` for mastodon.

## Feedback and contribution

If you noticed some bugs or are interested in new features just create an issue. You can also contribute to this repository by forking this repository and creating a pull request. 