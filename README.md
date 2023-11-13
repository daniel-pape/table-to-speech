# Table-to-speech

## Overview

A tool to create MP3 files using AWS Polly from your data base table.
Works with any table which provides the following three columns:

* id column
* column with the text you want as speech
* another column with the timestamp for the last update to the text

The created MP3 files can either be stored in S3 or in your local files.

Here is a bird's eye overview:

![Overview.png](doc%2FOverview.png)

1. Tool connects via SSH tunnel
2. Tool reads rows with text from specified data base
3. Text is converted to speech
4. Speech MP3s are either stored in S3 or locally

## How to run

### Prerequisites

* [Task: task runner](https://taskfile.dev/installation/) in order to work with `Taskfile.yml`
* Working AWS setup
 
### Steps

The following steps describe how you can export your table
to MP3 files created by Polly either to S3 or to your local file system.

It is assumed that your database runs in a private network and you have
a jump host which allows you to connect via SSH.

In the following steps `$PROJECT_ROOT` always stands for the path
to the root folder of this project.

**First step (setting up an SSH tunnel):**
1. Open a terminal
2. Copy the file `$PROJECT_ROOT/conf/ssh.conf.env.template` to `$PROJECT_ROOT/conf/ssh.conf.env` 
   and add the required information.
4. Run `task ssh-tunnel` to establish a SSH tunnel
5. Leave the terminal open

**Second step (running the actual export):**
1. Open another terminal
2. Create copies of the provided configuration templates files and provide the required information.
   Do this for:
   * `$PROJECT_ROOT/templates/connection.conf.env.template`
   * `$PROJECT_ROOT/templates/aws.conf.env.template`
   * `$PROJECT_ROOT/templates/file.export.conf.env.template`
   * `$PROJECT_ROOT/templates/s3.export.conf.env.template`
   
   For example copy `$PROJECT_ROOT/templates/connection.conf.env.template` to `$PROJECT_ROOT/conf/connection.conf.env` and
   fill in the required information. Proceed tge same way for the other files.
3. Run the export `task s3-export` if you want to export the MP3 created by Polly to S3 or run `task file-export`
   if you want to download MP3 files.
