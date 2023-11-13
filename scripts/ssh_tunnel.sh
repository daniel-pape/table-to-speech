#!/usr/bin/env bash
source ./conf/ssh.conf.env
ssh -L $LOCAL_PORT:$DATABASE_HOST:$DATABASE_PORT $SSH_USERNAME@$SSH_HOST -i $PATH_TO_SSH_PRIVATE_KEY -N
