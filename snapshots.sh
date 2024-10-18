#!/usr/bin/env bash

set -e

docker build --quiet --tag libreoffice .
docker run --volume ./examples:/examples libreoffice \
  sh -c '
    for f in /examples/* ; do
      cd "$f"
      /usr/bin/soffice --headless --convert-to pdf --outdir . ./output.docx;
      magick mogrify -path . -alpha remove -density 200 -format png ./output.pdf;
      # rm ./output.pdf
    done
'
