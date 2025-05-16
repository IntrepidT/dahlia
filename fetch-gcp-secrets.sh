#!/bin/bash

# Script to load multiple GCP secrets as environment variables before executing a command
# Usage: ./load-secrets.sh docker run your-image

set -e  # Exit immediately if a command exits with a non-zero status

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration - define your secrets here
# Format: SECRET_NAME:ENV_VAR_NAME
SECRETS=(
  "DATABASE_URL:DATABASE_URL"
  "FROM_EMAIL:FROM_EMAIL"
  "SMTP_PASSWORD:SMTP_PASSWORD"
  # Add more secrets as needed following the pattern "secret_name:ENV_VAR_NAME"
)

# Optional: Set your GCP project ID (uncomment if needed)
# PROJECT_ID="your-gcp-project-id"
# PROJECT_FLAG="--project=$PROJECT_ID"

echo -e "${BLUE}[INFO]${NC} Loading secrets from GCP Secret Manager..."

# Loop through all defined secrets and export them as environment variables
for SECRET_MAPPING in "${SECRETS[@]}"; do
  # Split the mapping into secret name and environment variable name
  SECRET_NAME=$(echo $SECRET_MAPPING | cut -d: -f1)
  ENV_VAR_NAME=$(echo $SECRET_MAPPING | cut -d: -f2)
  
  echo -e "${BLUE}[INFO]${NC} Fetching secret: $SECRET_NAME as $ENV_VAR_NAME"
  
  # Get the secret value, with optional project flag if PROJECT_ID is set
  if [ -n "${PROJECT_ID+x}" ]; then
    SECRET_VALUE=$(gcloud secrets versions access latest --secret="$SECRET_NAME" $PROJECT_FLAG)
  else
    SECRET_VALUE=$(gcloud secrets versions access latest --secret="$SECRET_NAME")
  fi
  
  if [ $? -ne 0 ]; then
    echo -e "${RED}[ERROR]${NC} Failed to fetch secret: $SECRET_NAME"
    exit 1
  fi
  
  # Export the secret as an environment variable
  export "$ENV_VAR_NAME"="$SECRET_VALUE"
done

echo -e "${GREEN}[SUCCESS]${NC} All secrets loaded as environment variables"

# Execute the command passed to the script with all environment variables
echo -e "${BLUE}[INFO]${NC} Executing command: $@"
exec "$@"
