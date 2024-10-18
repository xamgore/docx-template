FROM alpine:latest

# Install LibreOffice and fontconfig
RUN apk add --no-cache msttcorefonts-installer fontconfig && \
    update-ms-fonts && \
    fc-cache -f -v && \
    apk add --no-cache libreoffice libreoffice-common libpng-dev imagemagick imagemagick-pdf && \
    rm -rf /var/cache/apk/*
