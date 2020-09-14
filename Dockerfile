FROM node:12
# docker build -t stencila/schema .
WORKDIR /code

# Install app dependencies
COPY package*.json ./
COPY .Rprofile /root/.Rprofile

RUN apt-get update && \
    apt-get install -y python3-pip r-base && \
    # RScript -e "devtools::install_github('r-lib/systemfonts')" && \
    # this can be uncommented and changed to make setup if R deps can be resolved
    npm install && \
    npm audit fix

# Bundle app source
COPY . .
