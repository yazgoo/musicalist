#!/bin/env bash
set -x
(
(
(
curl https://en.wikipedia.org/wiki/List_of_musicals:_A_to_L ; 
curl https://en.wikipedia.org/wiki/List_of_musicals:_M_to_Z 
) \
   | grep '<td><i><a href' \
   | sed -E 's/.*title="([^"]*)"[^>]*>([^<]*).*/\2\t\1/' \
   | nl -w1 "-s$(echo -e '\t')"
) \
) | tee src/musicals.csv
(
echo 'include!("model.rs");'
echo 'pub static MUSICALS: once_cell::sync::Lazy<Vec<Musical>> = once_cell::sync::Lazy::new(|| {'
echo "vec!["
   cat src/musicals.csv | sed -E 's/([0-9]+)\t([^\t]*)\t([^\t]*)/Musical { id: \1, name: "\2".to_string(), url: "\3".to_string() },/'
echo "]"
echo "});"
) | tee src/musicals.rs
