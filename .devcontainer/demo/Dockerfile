FROM --platform=linux/amd64 ubuntu:24.04 

ENV DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC

RUN apt-get update \
 && apt-get install -y --allow-change-held-packages --no-install-recommends \
      build-essential git \
      r-base r-cran-tidyverse \
      python3 python3-pip python3-numpy python3-matplotlib python3-requests python3-pandas python3-setuptools python3-dev \ 
 && rm -rf /var/lib/apt/lists/* \
 && apt-get clean
