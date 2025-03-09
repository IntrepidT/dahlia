#!/bin/bash

# PostgreSQL Database Restore Script
# This script restores a PostgreSQL database from a pg_dump file

# Default values
DUMP_FILE=""
DB_NAME=""
DB_USER="postgres"
DB_HOST="localhost"
DB_PORT="5432"
VERBOSE=false
CLEAN=false
CREATE=false
NO_OWNER=false
NO_PRIVILEGES=false
NO_TABLESPACES=false

# Function to display usage information
function show_usage {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -f, --file FILENAME      Specify dump file to restore (required)"
    echo "  -d, --dbname DBNAME      Specify database name to restore to (required)"
    echo "  -U, --username USERNAME  Database user name (default: postgres)"
    echo "  -h, --host HOSTNAME      Database server host (default: localhost)"
    echo "  -p, --port PORT          Database server port (default: 5432)"
    echo "  -v, --verbose            Output verbose messages"
    echo "  -c, --clean              Clean (drop) database objects before recreating"
    echo "  -C, --create             Create the database before restoring"
    echo "  --no-owner               Skip restoration of object ownership"
    echo "  --no-privileges          Skip restoration of access privileges (grant/revoke)"
    echo "  --no-tablespaces         Skip restoration of tablespace assignments"
    echo "  --help                   Show this help"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -f|--file)
            DUMP_FILE="$2"
            shift 2
            ;;
        -d|--dbname)
            DB_NAME="$2"
            shift 2
            ;;
        -U|--username)
            DB_USER="$2"
            shift 2
            ;;
        -h|--host)
            DB_HOST="$2"
            shift 2
            ;;
        -p|--port)
            DB_PORT="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        -C|--create)
            CREATE=true
            shift
            ;;
        --no-owner)
            NO_OWNER=true
            shift
            ;;
        --no-privileges)
            NO_PRIVILEGES=true
            shift
            ;;
        --no-tablespaces)
            NO_TABLESPACES=true
            shift
            ;;
        --help)
            show_usage
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            ;;
    esac
done

# Check required parameters
if [ -z "$DUMP_FILE" ]; then
    echo "Error: Dump file not specified"
    show_usage
fi

if [ -z "$DB_NAME" ]; then
    echo "Error: Database name not specified"
    show_usage
fi

# Check if dump file exists
if [ ! -f "$DUMP_FILE" ]; then
    echo "Error: Dump file '$DUMP_FILE' does not exist"
    exit 1
fi

# Prepare pg_restore command
RESTORE_CMD="pg_restore -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME"

# Add optional flags
if [ "$VERBOSE" = true ]; then
    RESTORE_CMD="$RESTORE_CMD -v"
fi

if [ "$CLEAN" = true ]; then
    RESTORE_CMD="$RESTORE_CMD -c"
fi

if [ "$CREATE" = true ]; then
    RESTORE_CMD="$RESTORE_CMD -C"
fi

if [ "$NO_OWNER" = true ]; then
    RESTORE_CMD="$RESTORE_CMD -O"
fi

if [ "$NO_PRIVILEGES" = true ]; then
    RESTORE_CMD="$RESTORE_CMD -x"
fi

if [ "$NO_TABLESPACES" = true ]; then
    RESTORE_CMD="$RESTORE_CMD -T"
fi

# Add file name to restore command
RESTORE_CMD="$RESTORE_CMD $DUMP_FILE"

# Handle compressed files
if [[ "$DUMP_FILE" == *.gz ]]; then
    echo "Detected gzipped dump file, using gunzip pipe"
    RESTORE_CMD="gunzip -c $DUMP_FILE | pg_restore -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME"
    
    # Add optional flags for piped command
    if [ "$VERBOSE" = true ]; then
        RESTORE_CMD="$RESTORE_CMD -v"
    fi
    
    if [ "$CLEAN" = true ]; then
        RESTORE_CMD="$RESTORE_CMD -c"
    fi
    
    if [ "$CREATE" = true ]; then
        RESTORE_CMD="$RESTORE_CMD -C"
    fi
    
    if [ "$NO_OWNER" = true ]; then
        RESTORE_CMD="$RESTORE_CMD -O"
    fi
    
    if [ "$NO_PRIVILEGES" = true ]; then
        RESTORE_CMD="$RESTORE_CMD -x"
    fi
    
    if [ "$NO_TABLESPACES" = true ]; then
        RESTORE_CMD="$RESTORE_CMD -T"
    fi
elif [[ "$DUMP_FILE" == *.sql ]]; then
    echo "Detected SQL plain text dump, using psql instead of pg_restore"
    RESTORE_CMD="psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f $DUMP_FILE"
    
    if [ "$VERBOSE" = true ]; then
        RESTORE_CMD="$RESTORE_CMD -v"
    fi
fi

# Confirm before executing
echo "About to execute:"
echo "$RESTORE_CMD"
echo "Do you want to continue? [y/N]"
read CONFIRM

if [[ "$CONFIRM" == [yY] || "$CONFIRM" == [yY][eE][sS] ]]; then
    echo "Restoring database..."
    
    # Execute the restore command
    eval $RESTORE_CMD
    
    # Check if restore was successful
    if [ $? -eq 0 ]; then
        echo "Database restore completed successfully."
    else
        echo "Error: Database restore failed."
        exit 1
    fi
else
    echo "Database restore cancelled."
    exit 0
fi
