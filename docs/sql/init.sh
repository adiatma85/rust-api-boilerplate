#!/bin/bash

set -euxo pipefail

HOST="${MYSQL_HOST:-127.0.0.1}"
PORT="${MYSQL_PORT:-3306}"
DB_NAME="${MYSQL_DB:-kanban_db}"
USERNAME="${MYSQL_USER:-root}"
PASSWORD="${MYSQL_PASSWORD:-password}"
PROTOCOL="${MYSQL_PROTOCOL:-tcp}"

rm temp.sql || true

# REMOVED the "\n" inside the strings below
echo "DROP DATABASE IF EXISTS ${DB_NAME};" >> temp.sql
echo "CREATE DATABASE IF NOT EXISTS ${DB_NAME};" >> temp.sql
echo "USE ${DB_NAME};" >> temp.sql
echo "SET foreign_key_checks = 0;" >> temp.sql

# Note: The globs ** are expanded by bash.
cat **/[!temp]*.sql >> temp.sql || true

echo "SET foreign_key_checks = 1;" >> temp.sql

mysql -h ${HOST} -P ${PORT} -u ${USERNAME} --password=${PASSWORD} --protocol=${PROTOCOL} < temp.sql
rm temp.sql
