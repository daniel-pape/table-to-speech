#!/usr/bin/env bash
source ./conf/connection.conf.env
source ./conf/aws.conf.env
source ./conf/file.export.conf.env

DB_USER_NAME=$DB_USER_NAME \
DB_PASSWORD=$DB_PASSWORD \
DATABASE_NAME=$DATABASE_NAME \
cargo run -- --profile $AWS_PROFILE \
--region $AWS_REGION \
file-export \
--output-dir $OUTPUT_DIR \
--last-pollified $LAST_POLLIFIED \
--table-name $TABLE_NAME \
--id-column $ID_COLUMN \
--last-updated-column $LAST_UPDATED_COLUMN \
--text-column $TEXT_COLUMN
