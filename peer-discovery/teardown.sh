#!/bin/bash

# Function to check the status of the last command and exit if it failed
check_status() {
    if [ $? -ne 0 ]; then
        echo "Error: $1 failed"
        exit 1
    fi
}

echo "Deleting peer service..."
kubectl delete service peer-service -n bootstrap
check_status "Deleting peer service"

echo "Deleting bootstrap service..."
kubectl delete service bootstrap-service -n bootstrap
check_status "Deleting bootstrap service"

echo "Deleting peer deployment..."
kubectl delete deployment peer-deployment -n bootstrap
check_status "Deleting peer deployment"

echo "Deleting bootstrap deployment..."
kubectl delete deployment bootstrap-deployment -n bootstrap
check_status "Deleting bootstrap deployment"

echo "Teardown completed successfully!"
