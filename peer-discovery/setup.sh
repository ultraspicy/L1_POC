#!/bin/bash

# Function to check the status of the last command and exit if it failed
check_status() {
    if [ $? -ne 0 ]; then
        echo "Error: $1 failed"
        exit 1
    fi
}

echo "Applying namespace..."
kubectl apply -f k8s/namespace.yaml
check_status "Namespace creation"

echo "Applying role..."
kubectl apply -f k8s/role.yaml
check_status "Role creation"

echo "Applying rolebinding..."
kubectl apply -f k8s/rolebinding.yaml
check_status "RoleBinding creation"

# for bootstrap node 
echo "Applying bootstrap service..."
kubectl apply -f k8s/service-bootstrap.yaml
check_status "Bootstrap service creation"

echo "Applying bootstrap deployment..."
kubectl apply -f k8s/deployment-bootstrap-node.yaml
check_status "Bootstrap deployment creation"

# for peer node
echo "Applying peer service..."
kubectl apply -f k8s/service-peer.yaml
check_status "Peer service creation"

echo "Applying peer deployment..."
kubectl apply -f k8s/deployment-peer-node.yaml
check_status "Peer deployment creation"

echo "Setup completed successfully!"
