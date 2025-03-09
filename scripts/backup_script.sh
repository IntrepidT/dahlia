#!/bin/bash

#Set variables for database connection
PGUSER=postgres
PGDATABASE=dahlia

#Set the path where backup files are to be stored
BACKUP_DIR=/dahlia/backup_data


#Get current data and time 
datestamp=$(date +'%Y-%m-%d')
timestamp=$(date +'%H%M')

#Execute pg_dump command to dump the database
pg_dump -U "$PGUSER" -d "$PGDATABASE" > "BACKUP_DIR/$PGDATABASE"_"$datestamp"_"$timestamp".sql
