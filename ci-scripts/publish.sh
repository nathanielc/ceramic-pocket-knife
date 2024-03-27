#!/usr/bin/env bash

# Build and publish a docker image run running cpk
#
# DOCKER_PASSWORD must be set
# Use:
#
#   export DOCKER_PASSWORD=$(aws ecr-public get-login-password --region us-east-1)
#   echo "${DOCKER_PASSWORD}" | docker login --username AWS --password-stdin public.ecr.aws/r5b3e0r5
#
# to login to docker. That password will be valid for 12h.

docker buildx build --load -t 3box/cpk .
docker tag 3box/cpk:latest public.ecr.aws/r5b3e0r5/3box/cpk:latest
docker push -a public.ecr.aws/r5b3e0r5/3box/cpk