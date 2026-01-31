#!/bin/bash

# Exit on error
set -e

APP_NAME="ghostlink-v2-backend"
REGION="us-central1" # You can change this
PROJECT_ID=$(gcloud config get-value project)

if [ -z "$PROJECT_ID" ]; then
    echo "Error: No Google Cloud project selected."
    echo "Run 'gcloud config set project [YOUR_PROJECT_ID]' first."
    exit 1
fi

echo "Deploying to Project: $PROJECT_ID"

# 1. Build and submit image to Artifact Registry (using Cloud Build)
# This avoids needing local docker setup and uploading huge contexts if configured right
echo "Building container image..."
gcloud builds submit --tag gcr.io/$PROJECT_ID/$APP_NAME .

# 2. Deploy to Cloud Run
echo "Deploying to Cloud Run..."
gcloud run deploy $APP_NAME \
    --image gcr.io/$PROJECT_ID/$APP_NAME \
    --platform managed \
    --region $REGION \
    --allow-unauthenticated \
    --memory 4Gi \
    --cpu 2

# 3. Output URL
echo "Deployment Complete!"
SERVICE_URL=$(gcloud run services describe $APP_NAME --platform managed --region $REGION --format 'value(status.url)')
echo "Service URL: $SERVICE_URL"
echo "Please update your frontend configuration with this URL."
