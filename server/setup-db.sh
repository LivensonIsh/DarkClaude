#!/bin/bash
set -e
cd ~/DarkClaude/server
DBPASS=$(openssl rand -hex 16)
sudo -u postgres psql -c "DROP DATABASE IF EXISTS darkclaude_db;" < /dev/null
sudo -u postgres psql -c "DROP USER IF EXISTS darkclaude_user;" < /dev/null
sudo -u postgres psql -c "CREATE USER darkclaude_user WITH PASSWORD '$DBPASS';" < /dev/null
sudo -u postgres psql -c "CREATE DATABASE darkclaude_db OWNER darkclaude_user TEMPLATE template0 ENCODING 'UTF8';" < /dev/null
sudo -u postgres psql -d darkclaude_db -c "GRANT ALL ON SCHEMA public TO darkclaude_user;" < /dev/null
sed -i "s|^DATABASE_URL=.*|DATABASE_URL=postgresql://darkclaude_user:$DBPASS@127.0.0.1:5432/darkclaude_db|" .env
chmod 600 .env
psql "$(grep ^DATABASE_URL .env | cut -d= -f2-)" -c "select 'BASE OK' as test;"
