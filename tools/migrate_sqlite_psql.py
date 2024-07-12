# Migrate the old sqlite database to new postgresql database
# required packages:
#   python-dotenv, psycopg2
# install with:
#   pip install python-dotenv psycopg2

from sys import exit
from os import environ
from dotenv import load_dotenv
import sqlite3
import psycopg2

MIGRATION_SQL = [
    "CREATE TABLE migration(id SMALLINT NOT NULL PRIMARY KEY)",
    "CREATE TABLE telegram_user (id SERIAL PRIMARY KEY, chat_id BIGINT NOT NULL UNIQUE, trigger_warnings TEXT[] DEFAULT array[]::varchar[] NOT NULL, show_planned_disruptions BOOLEAN DEFAULT FALSE NOT NULL)",
    "CREATE TABLE disruption (id SERIAL PRIMARY KEY, key TEXT NOT NULL UNIQUE, hash VARCHAR(32) NOT NULL, start_time TIMESTAMP WITHOUT TIME ZONE, end_time TIMESTAMP WITHOUT TIME ZONE, json JSONB)",
    "CREATE TABLE mastodon_toot (id SERIAL PRIMARY KEY, disruption_id INTEGER NOT NULL, status_id VARCHAR(255), CONSTRAINT disruption_id FOREIGN KEY (disruption_id) REFERENCES disruption(id) ON DELETE CASCADE)",
]

if __name__ == "__main__":
    load_dotenv()
    if "SQLITE_PATH" not in environ:
        print("Environment variable 'SQLITE_PATH' not set.")
        exit(1)
    if "POSTGRESQL_CONFIG" not in environ:
        print("Environment variablev 'POSTGRESQL_CONFIG' not set.")
        exit(1)

    sqlite_connection = sqlite3.connect(environ["SQLITE_PATH"])
    postgres_connection = psycopg2.connect(environ["POSTGRESQL_CONFIG"])

    s_cursor = sqlite_connection.cursor()
    p_cursor = postgres_connection.cursor()

    # check if migration table exists
    p_cursor.execute(
        "SELECT EXISTS (SELECT * FROM information_schema.tables WHERE table_name='migration')"
    )
    if p_cursor.fetchone()[0]:
        print(
            "Migration table already exists. Running this script on a non-empty database does not work."
        )
        exit(1)

    print("Creating empty tables...")
    for sql in MIGRATION_SQL:
        p_cursor.execute(sql)
    p_cursor.executemany("INSERT INTO migration(id) VALUES(%s)", ((1,), (2,)))

    print("Migrating telegram users...")
    s_cursor.execute(
        "SELECT chat_id, trigger_warning_list, show_planned_disruptions FROM user"
    )
    row = s_cursor.fetchone()
    while row is not None:
        p_cursor.execute(
            "INSERT INTO telegram_user(chat_id, trigger_warnings, show_planned_disruptions) VALUES(%s, %s, %s)",
            (row[0], [] if row[1] == "" else list(row[1].split(",")), bool(row[2])),
        )
        row = s_cursor.fetchone()

    print("Migrating disruptions...")
    s_cursor.execute("SELECT key, hash, start_time, end_time FROM disruption")
    row = s_cursor.fetchone()
    while row is not None:
        p_cursor.execute(
            "INSERT INTO disruption(key, hash, start_time, end_time) VALUES(%s, %s, %s, %s)",
            row,
        )
        row = s_cursor.fetchone()

    print("Migrating mastodon toots...")
    s_cursor.execute(
        "SELECT disruption.key, toots.status_id FROM toots JOIN disruption ON toots.disruption_id=disruption.id"
    )
    row = s_cursor.fetchone()
    while row is not None:
        p_cursor.execute(
            "INSERT INTO mastodon_toot(disruption_id, status_id) VALUES((SELECT id FROM disruption WHERE key=%s), %s)",
            row,
        )
        row = s_cursor.fetchone()

    print("Migration finished")
    postgres_connection.commit()
    s_cursor.close()
    p_cursor.close()
    sqlite_connection.close()
    postgres_connection.close()
