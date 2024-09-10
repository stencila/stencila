FROM node:latest

# Export config paths for NPM
ENV NPM_CONFIG_PREFIX=/home/node/.npm-global
ENV PATH=$PATH:/home/node/.npm-global/bin

# Set up the app folder + parcel cache & set permissions
RUN mkdir -p /home/node/app/.parcel-cache && chown -R node:node /home/node/app
RUN chmod 755 /home/node/app/.parcel-cache

# set the workdir to the new folder, copy pacakge so we can install the
# dependencies.
WORKDIR /home/node/app
COPY --chown=node:node package.json .

# Make sure we run as node now.
USER node

# Install the dependencies
RUN npm i

# copy the rest of the app
COPY --chown=node:node . .
