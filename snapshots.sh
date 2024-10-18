#!/usr/bin/env bash

# japanese: https://github.com/yuntara/docx-png-converter/tree/master

set -e

docker build --quiet --tag libreoffice .
docker run --rm --name docx-to-png --volume ./examples:/examples libreoffice \
  sh -c '
    for f in /examples/* ; do
      cd "$f"
      /usr/bin/soffice --headless --convert-to pdf --outdir . ./output.docx;
      magick mogrify -path . -alpha remove -density 200 -format png +append ./output.pdf;
      rm ./output.pdf
    done
'
